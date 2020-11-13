import logging
from dataclasses import dataclass
from datetime import datetime
from threading import Thread

from serial import Serial

from energymonitor.services.dispatcher import pubsub


@dataclass
class Measurements:
    ADCO: str  # electric meter address
    PTEC: str  # current tariff period
    HCHC: int  # heures creuses index, in watts
    HCHP: int  # heures pleines index, in watts
    timestamp: datetime = None


class Linky(Thread):
    """
    Service responsible for reading and emitting measurements read from Linky.
    See https://www.enedis.fr/sites/default/files/Enedis-NOI-CPT_02E.pdf
    """

    def __init__(self):
        super().__init__(name=self.__class__.__name__)
        self.logger = logging.getLogger(self.__class__.__name__)
        self.serial = Serial(port='/dev/ttyUSB0', baudrate=1200)
        self.logger.debug('Initialized')

    def run(self):
        HCHC, HCHP = 0, 0
        import random
        from time import sleep
        while True:
            sleep(2)
            HCHC += random.randint(0, 10)
            HCHP += random.randint(0, 10)
            pubsub.publish(Measurements(ADCO='12345', PTEC='HC..', HCHC=HCHC, HCHP=HCHP, timestamp=datetime.utcnow()))
        ######
        buffer = {}
        self.serial.readline()  # Ignore (potentially incomplete) first line
        while True:
            line = self.serial.readline().decode()
            (key, value) = line.strip().split()[0:1]
            # publish if we hit a new frame
            if key == 'ADCO':
                try:
                    measurements = Measurements(timestamp=datetime.utcnow(), **buffer)
                    self.logger.debug('Read from Linky: %s', measurements)
                    pubsub.publish(measurements)
                    buffer = {}
                except TypeError:
                    self.logger.debug('Incomplete frame from Linky')
                    buffer = {}
            buffer[key] = value
