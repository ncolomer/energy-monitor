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
        'RPi.GPIO~=0.7.0',
        'gpiozero~=1.5.1',
        'spidev~=3.5',
        'pyserial~=3.4',
        'Pillow~=7.2.0',
        'influxdb~=5.3.0',
    ],
)
