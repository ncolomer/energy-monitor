import logging
from collections import deque

from energymonitor.devices import button, rpict, linky
from energymonitor.devices.button import Button
from energymonitor.devices.display import Display
from energymonitor.services.dispatcher import pubsub
from energymonitor.services.pages import LandingPage, RPICTPage, LinkyPage
from energymonitor.services import datalogger


class Interface:
    """
    Class responsible for intercepting and displaying pages.
    See https://www.waveshare.com/wiki/2.23inch_OLED_HAT
    """

    def __init__(self):
        self.logger = logging.getLogger(self.__class__.__name__)
        # create devices
        self.button = Button()
        self.display = Display()
        # create pages and display landing
        self.landing_page = LandingPage(size=self.display.size())
        self.rpict_page = RPICTPage(size=self.display.size())
        self.linky_page = LinkyPage(size=self.display.size())
        self.carousel = deque([self.landing_page, self.linky_page, self.rpict_page])
        self.refresh_display()
        # subscribe to events
        pubsub.subscribe(self.__class__.__name__, self.handle_message)
        self.logger.debug('Initialized')

    def handle_message(self, message):
        if type(message) == rpict.Ready:
            self.landing_page.refresh(rpict=True)
            self.refresh_display(page=self.landing_page)
        elif type(message) == linky.Ready:
            self.landing_page.refresh(linky=True)
            self.refresh_display(page=self.landing_page)
        elif type(message) == datalogger.Ready:
            self.landing_page.refresh(influxdb=True)
            self.refresh_display(page=self.landing_page)
        elif type(message) == rpict.Measurements:
            self.rpict_page.refresh(message)
            self.refresh_display(page=self.rpict_page)
        elif type(message) == linky.Measurements:
            self.linky_page.refresh(message)
            self.refresh_display(page=self.linky_page)
        elif type(message) == button.WakeupEvent:
            self.logger.info('Received WakeupEvent')
            self.carousel = deque([self.rpict_page, self.landing_page, self.linky_page])
            self.refresh_display()
            self.display.on()
        elif type(message) == button.InactivityEvent:
            self.logger.info('Received InactivityEvent')
            self.display.off()
        elif type(message) == button.PressEvent:
            self.logger.info('Received PressEvent')
            self.carousel.rotate(1)
            self.refresh_display()

    def refresh_display(self, page=None):
        if not page or self.carousel[0] == page:
            self.display.print(self.carousel[0].image())

    def stop(self):
        self.display.clear()
        self.display.off()
