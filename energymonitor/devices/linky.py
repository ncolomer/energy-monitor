import inspect
import logging
from dataclasses import dataclass
from datetime import datetime
from threading import Thread

from serial import Serial, PARITY_EVEN

from energymonitor.config import LINKY_SERIAL_PORT
from energymonitor.services.dispatcher import pubsub


@dataclass
class Ready:
    pass


@dataclass
class Measurements:
    ADCO: str  # electric meter address
    PTEC: str  # current tariff period
    HCHC: int  # heures creuses index, in watts
    HCHP: int  # heures pleines index, in watts
    timestamp: datetime = None

    @classmethod
    def from_dict(cls, d):
        return cls(**{
            k: v for k, v in d.items()
            if k in inspect.signature(cls).parameters
        })


class Linky(Thread):
    """
    Service responsible for reading and emitting measurements read from Linky.
    See https://www.enedis.fr/sites/default/files/Enedis-NOI-CPT_02E.pdf
    """

    def __init__(self):
        super().__init__(name=self.__class__.__name__)
        self.logger = logging.getLogger(self.__class__.__name__)
        self.serial = Serial(port=LINKY_SERIAL_PORT, baudrate=1200, bytesize=7, parity=PARITY_EVEN)
        pubsub.publish(Ready())
        self.logger.debug('Initialized')

    def run(self):
        buffer = {}
        self.serial.readline()  # Ignore (potentially incomplete) first line
        while True:
            # read and parse line
            line = self.serial.readline().decode()
            (key, value) = line.strip().split()[0:2]
            if key in ('HCHC', 'HCHP'): value = int(value)
            # publish if we hit a new frame
            if key == 'ADCO':
                try:
                    buffer['timestamp'] = datetime.utcnow()
                    measurements = Measurements.from_dict(buffer)
                    self.logger.debug('Read from Linky: %s', measurements)
                    pubsub.publish(measurements)
                    buffer = {}
                except TypeError as exc:
                    self.logger.debug('Incomplete frame from Linky: %s', buffer, exc_info=exc)
                    buffer = {}
            buffer[key] = value
