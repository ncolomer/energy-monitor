import logging

from influxdb import InfluxDBClient

from energymonitor.config import INFLUX_DB_HOST, INFLUX_DB_PORT, INFLUX_DB_DATABASE, INFLUX_DB_PREFIX
from energymonitor.devices import linky, rpict
from energymonitor.services.dispatcher import pubsub


class DataLogger:
    """
    Class responsible for intercepting and pushing measurements to influxdb.
    """

    def __init__(self) -> None:
        self.logger = logging.getLogger(self.__class__.__name__)
        self.influx = InfluxDBClient(host=INFLUX_DB_HOST, port=INFLUX_DB_PORT, database=INFLUX_DB_DATABASE)
        influxdb_version = self.influx.ping()  # checking connectivity
        self.logger.debug('Connected to InfluxDB %s', influxdb_version)
        pubsub.subscribe(self.__class__.__name__, self.handle_message)
        self.logger.debug('Initialized')

    def handle_message(self, message):
        if type(message) == rpict.Measurements:
            self.influx.write_points([{
                'measurement': f'{INFLUX_DB_PREFIX}.rpict',
                'time': message.timestamp.isoformat() + 'Z',
                'fields': {
                    'l1_real_power': message.l1_real_power,
                    'l1_apparent_power': message.l1_apparent_power,
                    'l1_irms': message.l1_irms,
                    'l1_vrms': message.l1_vrms,
                    'l1_power_factor': message.l1_power_factor,

                    'l2_real_power': message.l2_real_power,
                    'l2_apparent_power': message.l2_apparent_power,
                    'l2_irms': message.l2_irms,
                    'l2_vrms': message.l2_vrms,
                    'l2_power_factor': message.l2_power_factor,

                    'l3_real_power': message.l3_real_power,
                    'l3_apparent_power': message.l3_apparent_power,
                    'l3_irms': message.l3_irms,
                    'l3_vrms': message.l3_vrms,
                    'l3_power_factor': message.l3_power_factor,
                }
            }])
        elif type(message) == linky.Measurements:
            self.influx.write_points([{
                'measurement': f'{INFLUX_DB_PREFIX}.linky',
                'time': message.timestamp.isoformat() + 'Z',
                'fields': {
                    'hc_index': message.HCHC,
                    'hp_index': message.HCHP,
                }
            }])
