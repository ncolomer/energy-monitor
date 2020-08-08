import logging

from PIL import Image

import energymonitor.devices.ssd1305.SPI as SPI
import energymonitor.devices.ssd1305.SSD1305 as SSD1305
from energymonitor.helpers.constants import LOGO


class Display:
    """
    Service responsible for communication with the display.
    See https://www.waveshare.com/wiki/2.23inch_OLED_HAT
    """

    def __init__(self):
        self.logger = logging.getLogger(self.__class__.__name__)
        spi = SPI.SpiDev(port=0, device=0, max_speed_hz=8000000)
        self._display = SSD1305.SSD1305_128_32(rst=None, dc=24, spi=spi)
        # Initialize sequence
        self._display.begin()
        self._display.set_contrast(0)
        self.print(LOGO)
        self.logger.info('Initialized')

    def image(self):
        return Image.new(mode='1', size=(self._display.width, self._display.height), color=0)

    def print(self, image):
        self._display.image(image)
        self._display.display()

    def display_on(self):
        self._display.command(SSD1305.SSD1305_DISPLAYON)

    def display_off(self):
        self._display.command(SSD1305.SSD1305_DISPLAYOFF)

    def display_clear(self):
        self._display.clear()
        self._display.display()
