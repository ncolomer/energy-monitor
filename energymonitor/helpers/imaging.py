from PIL import Image, ImageDraw

from energymonitor.helpers.constants import FONT
from energymonitor.helpers.maths import clamp


def clear(image: Image):
    draw = ImageDraw.Draw(image)
    draw.rectangle([(0, 0), image.size], fill=0)


def add_text(image: Image, xy: (int, int), text: str):
    draw = ImageDraw.Draw(image)
    draw.text(xy=xy, text=text, font=FONT, fill=255)


def add_bar(image: Image, xy: (int, int), value: float, max: float = None):
    draw = ImageDraw.Draw(image)
    # border
    draw.rectangle([(xy[0], xy[1] + 1),
                    (image.width - 1, xy[1] + 6)],
                   outline=255, fill=0)
    # value
    bar_width = image.width - xy[0] - 5
    bar_height = 1
    bar_start_x = xy[0] + 2
    bar_start_y = xy[1] + 3
    value_offset = int(bar_width * clamp(value, 0, 1))
    draw.rectangle([(bar_start_x, bar_start_y),
                    (bar_start_x + value_offset, bar_start_y + bar_height)],
                   outline=255, fill=255)
    # max
    if max is not None:
        max_value_offset = int(bar_width * clamp(max, 0, 1))
        draw.line([(bar_start_x + max_value_offset, bar_start_y),
                   (bar_start_x + max_value_offset, bar_start_y + bar_height)],
                  fill=255, width=1)
