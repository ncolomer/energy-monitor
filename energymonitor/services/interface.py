import logging

from energymonitor.devices import button, rpict
from energymonitor.devices.button import Button
from energymonitor.devices.display import Display
from energymonitor.services.dispatcher import pubsub
from energymonitor.services.pages import RPICTPage, LandingPage


class Interface:
    """
    Class responsible for intercepting and displaying pages.
    See https://www.waveshare.com/wiki/2.23inch_OLED_HAT
    """

    def __init__(self):
        self.logger = logging.getLogger(self.__class__.__name__)
        self.button = Button()
        self.display = Display()
        self.landing_page = LandingPage(size=self.display.size())
        self.display.print(self.landing_page.image())
        self.rpict_page = RPICTPage(size=self.display.size())
        pubsub.subscribe(self.__class__.__name__, self.handle_message)
        self.logger.debug('Initialized')

    def handle_message(self, message):
        if type(message) == rpict.Measurements:
            self.rpict_page.refresh(message)
            self.display.print(self.rpict_page.image())
        elif type(message) == button.InactivityEvent:
            self.logger.info('Received InactivityEvent')
            self.display.off()
        elif type(message) == button.PressEvent:
            self.logger.info('Received PressEvent')
            self.display.on()

    def stop(self):
        self.display.clear()
        self.display.off()
