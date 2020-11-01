from PIL import ImageFont, Image
from pkg_resources import resource_stream

MIN_POWER = 0
MAX_POWER = 230 * 30

FONT = ImageFont.truetype(resource_stream(__name__, 'data/ProggyTiny.ttf'), size=15)
LOGO = Image.open(resource_stream(__name__, 'data/logo.xbm'))

try:
    from importlib import metadata
except ImportError:
    import importlib_metadata as metadata
VERSION = metadata.version('energy-monitor')
