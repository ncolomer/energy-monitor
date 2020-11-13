from os import getenv

from setuptools import setup, find_packages

setup(
    name='energy-monitor',
    version=getenv('VERSION'),
    author='Nicolas Colomer',
    license='The Unlicense',
    description='Energy Monitor application',
    url='https://github.com/ncolomer/energy-monitor',
    packages=find_packages(include='energymonitor.*'),
    scripts=['scripts/energy-monitor'],
    package_data={
        "energymonitor.helpers": ["data/*"],
    },
    install_requires=[
        'importlib-metadata~=1.0;python_version<"3.8"',
        'influxdb~=5.3.0',
        # oled
        'adafruit-circuitpython-ssd1305~=1.3.3',
        'Pillow~=7.2.0',
        # gpio
        'RPi.GPIO~=0.7.0',
        # rpict + uteleinfo
        'pyserial~=3.4',
    ],
)
