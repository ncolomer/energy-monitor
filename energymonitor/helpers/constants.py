from PIL import ImageFont, Image, ImageChops, ImageDraw
from pkg_resources import resource_stream

MIN_POWER = 0
MAX_POWER = 230 * 30

try:
    from importlib import metadata
except ImportError:
    import importlib_metadata as metadata
VERSION = metadata.version('energy-monitor')

FONT = ImageFont.truetype(resource_stream(__name__, 'data/ProggyTiny.ttf'), size=15)


def load_logo() -> Image:
    logo = Image.open(resource_stream(__name__, 'data/logo.xbm'))
    logo = ImageChops.offset(logo, xoffset=0, yoffset=-5)
    draw = ImageDraw.Draw(logo)
    version = f'v{VERSION}'
    (font_width, font_height) = FONT.getsize(version)
    draw.text(
        xy=((128 - font_width) // 2, 32 - font_height - 1),
        text=version,
        font=FONT,
        fill=255,
    )
    return logo


LOGO = load_logo()
