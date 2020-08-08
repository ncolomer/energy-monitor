import logging
from dataclasses import dataclass
from datetime import datetime
from threading import Thread

from serial import Serial

from energymonitor.services.dispatcher import pubsub


@dataclass
class Measurements:
    node_id: int
    l1_real_power: float
    l1_apparent_power: float
    l1_irms: float
    l1_vrms: float
    l1_power_factor: float
    l2_real_power: float
    l2_apparent_power: float
    l2_irms: float
    l2_vrms: float
    l2_power_factor: float
    l3_real_power: float
    l3_apparent_power: float
    l3_irms: float
    l3_vrms: float
    l3_power_factor: float
    timestamp: datetime = None


class RPICT(Thread):
    """
    Service responsible for reading and emitting measurements read from RPICT.
    See http://lechacal.com/wiki/index.php/RPICT3V1
    See http://lechacal.com/wiki/index.php/Howto_setup_Raspbian_for_serial_read
    """

    def __init__(self):
        super().__init__(name=self.__class__.__name__)
        self.logger = logging.getLogger(self.__class__.__name__)
        self.serial = Serial(port='/dev/ttyAMA0', baudrate=38400)
        self.logger.info('Initialized')

    def run(self):
        self.serial.readline()  # Ignore (potentially incomplete) first line
        while True:
            items = self.serial.readline().decode().strip().split()
            measurements = Measurements(int(items[0]), *map(float, items[1:]), timestamp=datetime.utcnow())
            self.logger.debug('Read from RPICT: %s', measurements)
            pubsub.publish(measurements)
