from os import getenv

# human machine interface
HMI_SLEEP_SECS = int(getenv('HMI_SLEEP_SECS', '30'))
HMI_MAX_LINE_POWER_WATTS = int(getenv('HMI_MAX_LINE_POWER_WATTS', '6900'))  # 230V * 30A
HMI_BUTTON_DEBOUNCE_MS = int(getenv('HMI_BUTTON_DEBOUNCE_MS', '200'))

# drivers
RPICT_SERIAL_PORT = getenv('RPICT_SERIAL_PORT', '/dev/ttyAMA0')
LINKY_SERIAL_PORT = getenv('LINKY_SERIAL_PORT', '/dev/ttyUSB0')

# influx db
INFLUX_DB_HOST = getenv('INFLUX_DB_HOST', 'localhost')
INFLUX_DB_PORT = int(getenv('INFLUX_DB_PORT', '8086'))
INFLUX_DB_DATABASE = getenv('INFLUX_DB_DATABASE', 'metrology')
INFLUX_DB_PREFIX = getenv('INFLUX_DB_PREFIX', 'energy')

# misc
LOG_LEVEL = getenv('LOG_LEVEL', 'INFO').upper()
