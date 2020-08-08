import logging
from statistics import mean

from PIL import Image

from energymonitor.devices import button, rpict
from energymonitor.devices.button import Button
from energymonitor.devices.display import Display
from energymonitor.helpers.imaging import add_text, add_bar
from energymonitor.services.dispatcher import pubsub


class Interface:
    """
    Class responsible for intercepting and displaying pages.
    See https://www.waveshare.com/wiki/2.23inch_OLED_HAT
    """

    rpict_page: Image = None

    def __init__(self):
        self.logger = logging.getLogger(self.__class__.__name__)
        self.button = Button()
        self.display = Display()
        pubsub.subscribe(self.handle_message)
        self.logger.info('Initialized')

    def build_rpict_page(self, m: rpict.Measurements) -> Image:
        image = self.display.image()
        # line 1
        add_text(image, (0, 0), f'P1 {m.l1_apparent_power:4.0f}W')
        add_bar(image, 0, m.l1_apparent_power, m.l1_real_power)
        # line 2
        add_text(image, (0, 8), f'P2 {m.l2_apparent_power:4.0f}W')
        add_bar(image, 8, m.l2_apparent_power, m.l2_real_power)
        # line 3
        add_text(image, (0, 16), f'P3 {m.l3_apparent_power:4.0f}W')
        add_bar(image, 16, m.l3_apparent_power, m.l3_real_power)
        # line 4
        total_apparent_power = m.l1_apparent_power + m.l2_apparent_power + m.l3_apparent_power
        add_text(image, (0, 24), f'= {total_apparent_power / 1000:4.1f}kW')
        avg_vrms = mean([m.l1_vrms, m.l2_vrms, m.l3_vrms])
        add_text(image, (87, 24), f'{avg_vrms:5.2f}V')
        return image

    def handle_message(self, message):
        if type(message) == rpict.Measurements:
            self.rpict_page = self.build_rpict_page(message)
            self.display.print(self.rpict_page)
        elif type(message) == button.InactivityEvent:
            self.logger.info('Received InactivityEvent')
            self.display.display_off()
        elif type(message) == button.PressEvent:
            self.logger.info('Received PressEvent')
            self.display.display_on()
        elif type(message) == button.HeldEvent:
            self.logger.info('Received HeldEvent')

    def stop(self):
        self.display.display_clear()
        self.display.display_off()
