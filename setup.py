from os import getenv

from setuptools import setup, find_packages

setup(
    name='energy-monitor',
    version=getenv('VERSION'),
    description='Energy Monitor application',
    packages=find_packages(include='energymonitor.*'),
    scripts=['scripts/energy-monitor'],
    package_data={
        "energymonitor.helpers": ["data/*"],
    },
    install_requires=[
        'RPi.GPIO~=0.7.0',
        'gpiozero~=1.5.1',
        'spidev~=3.5',
        'pyserial~=3.4',
        'Pillow~=7.2.0',
        'influxdb~=5.3.0',
    ],
)
