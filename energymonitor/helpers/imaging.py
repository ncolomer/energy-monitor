from PIL import Image, ImageDraw

from energymonitor.helpers.constants import MAX_POWER, FONT
from energymonitor.helpers.maths import clamp


def add_text(image: Image, xy: (int, int), text: str):
    canvas = ImageDraw.Draw(image)
    canvas.text(xy, text, font=FONT, fill=255)


def add_bar(image: Image, height: int, value: float, max_value: float = None):
    canvas = ImageDraw.Draw(image)
    BAR_WIDTH = 76
    BAR_XS = image.width - BAR_WIDTH
    BAR_YS = height + 1
    canvas.rectangle([(BAR_XS, BAR_YS), (image.width - 1, BAR_YS + 6)], outline=255, fill=0)

    BAR_VAL_WIDTH = BAR_WIDTH - 5
    BAR_VAL_XS = BAR_XS + 2
    BAR_VAL_YS = BAR_YS + 2

    normalized_value = clamp(value, 0, MAX_POWER) / MAX_POWER
    xv = int(BAR_VAL_WIDTH * normalized_value)
    canvas.rectangle([(BAR_VAL_XS, BAR_VAL_YS), (BAR_VAL_XS + xv, BAR_VAL_YS + 2)], outline=255, fill=255)

    if max_value is not None:
        normalized_max_value = clamp(max_value, 0, MAX_POWER) / MAX_POWER
        xm = int(BAR_VAL_WIDTH * normalized_max_value)
        canvas.line([(BAR_VAL_XS + xm, BAR_VAL_YS), (BAR_VAL_XS + xm, BAR_VAL_YS + 2)], fill=255, width=1)
