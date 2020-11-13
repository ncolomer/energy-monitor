from PIL import ImageFont, Image, ImageDraw, ImageChops
from pkg_resources import resource_stream

MIN_POWER = 0
MAX_POWER = 230 * 30

try:
    from importlib import metadata
except ImportError:
    import importlib_metadata as metadata
VERSION = metadata.version('energy-monitor')

FONT = ImageFont.truetype(resource_stream(__name__, 'data/ProggyTiny.ttf'), size=15)


def load_logo():
    logo = Image.open(resource_stream(__name__, 'data/logo.xbm'))
    ImageChops.offset(logo, xoffset=0, yoffset=-5)
    version = f'v{VERSION}'
    (font_width, font_height) = FONT.getsize(version)
    ImageDraw.Draw(logo).text(xy=((logo.size[0] - font_width) // 2, logo.size[1] - font_height - 1),
                              text=version, font=FONT, fill=255)
    return logo


LOGO = load_logo()
