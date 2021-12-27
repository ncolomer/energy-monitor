import logging
import sys
from signal import signal, SIGTERM

from energymonitor import VERSION
from energymonitor.config import LOG_LEVEL
from energymonitor.devices.linky import Linky
from energymonitor.devices.rpict import RPICT
from energymonitor.services.datalogger import DataLogger
from energymonitor.services.interface import Interface

logging.basicConfig(stream=sys.stdout, level=LOG_LEVEL, format='%(name)s - %(levelname)s: %(message)s')
logger = logging.getLogger('Main')

try:
    rpict = RPICT()
    rpict.start()
except Exception as exc:
    logger.warning('Could not start RPICT driver', exc_info=exc)

try:
    linky = Linky()
    linky.start()
except Exception as exc:
    logger.warning('Could not start RPICT driver', exc_info=exc)

try:
    datalogger = DataLogger()
except Exception as exc:
    logger.warning('Could not start data logger', exc_info=exc)


interface = Interface()


def shutdown(signum, stack):
    interface.stop()
    sys.exit()


signal(SIGTERM, shutdown)

logger.info(f'energy-monitor {VERSION} started!')

