import logging

import adafruit_ssd1305
import board
import digitalio
from PIL import Image
from adafruit_ssd1305 import SET_DISP

from energymonitor.helpers.constants import LOGO


class Display:
    """
    Service responsible for communication with the display.
    See https://www.waveshare.com/wiki/2.23inch_OLED_HAT
    """

    def __init__(self):
        self.logger = logging.getLogger(self.__class__.__name__)

        self._oled = adafruit_ssd1305.SSD1305_SPI(
            width=128, height=32,
            spi=board.SPI(),
            dc=digitalio.DigitalInOut(board.D5),
            reset=digitalio.DigitalInOut(board.D4),
            cs=digitalio.DigitalInOut(board.D6)
        )

        # Initialize sequence
        self._oled.contrast(0xFF)
        self.print(LOGO)
        self.logger.info('Initialized')

    def image(self):
        return Image.new(mode='1', size=(self._oled.width, self._oled.height), color=0)

    def display_on(self):
        self._oled.write_cmd(SET_DISP | 0x01)

    def print(self, image):
        self._oled.image(image)
        self._oled.show()

    def display_clear(self):
        self._oled.fill(0)
        self._oled.show()

    def display_off(self):
        self._oled.write_cmd(SET_DISP | 0x00)
