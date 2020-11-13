import logging

import adafruit_ssd1305
import board
import digitalio
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
            dc=digitalio.DigitalInOut(board.D24),
            reset=digitalio.DigitalInOut(board.D25),
            cs=digitalio.DigitalInOut(board.CE0)
        )

        # Initialize sequence
        self._oled.contrast(0x00)
        self.print(LOGO)
        self.logger.debug('Initialized')

    def size(self) -> (int, int):
        return self._oled.width, self._oled.height

    def on(self):
        self._oled.write_cmd(SET_DISP | 0x01)

    def print(self, image):
        self._oled.image(image)
        self._oled.show()

    def clear(self):
        self._oled.fill(0)
        self._oled.show()

    def off(self):
        self._oled.write_cmd(SET_DISP | 0x00)
