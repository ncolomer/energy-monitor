# energy-monitor

> <img height="256" alt="energy-monitor module" src="https://user-images.githubusercontent.com/941891/99180442-a545fd00-2726-11eb-9eb4-781ce5c0c186.png"> <img height="256" alt="installed energy-monitor module" src="https://user-images.githubusercontent.com/941891/99187647-32557a00-2758-11eb-86f0-c68d863e7cef.png"> <img height="256" alt="connections of energy-monitor module" src="https://user-images.githubusercontent.com/941891/99187649-35e90100-2758-11eb-9a7a-da070b3239cc.png">

> You can't improve what you don't measure

This project is a DIY module + a Python application that aim at measuring electrical consumption metrics, display collected values on an OLED display, and send them to an external InfluxDB database for historization. 

The module was built to fit any standard electrical panel (same form factor as a circuit breaker). 
It does not collect data directly but rather fetches metrics from [Lechacal](http://lechacal.com/)'s [RPICT](http://lechacal.com/wiki/index.php?title=Raspberrypi_Current_and_Temperature_Sensor_Adaptor) module and Enedis [Linky](https://fr.wikipedia.org/wiki/Linky) electric meter (France national power provider).

I built this project to observe and store my own energy consumption, to eventually improve them. And also because it looked a cool DIY project (it actually was!).

:warning: My electrical installation has three-phase power supply. Even though the project can run for a one-phase power, it might also need some adaptation. Any contribution welcome!

![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/ncolomer/energy-monitor?label=latest%20release&sort=semver&style=for-the-badge)

## Application

This project includes a Python application that handles everything, from I/O to display.

### Interface

#### Startup screen

This screen displays the project logo and the current application version.
It is shown at application startup and also belongs to the page carousel (last position).

> <img height="96" alt="landing screen" src="https://user-images.githubusercontent.com/941891/99180029-04097780-2723-11eb-8937-fbdda2956b72.png">

#### Instantaneous metrics screen (RPICT)

This screen displays instantaneous metrics measured from RPICT:
- lines apparent power value with a gauge that shows the max seen value since boot
- sum of consumed lines power
- mean of lines RMS voltage

It is the first displayed screen when waking up from sleep.

> <img height="96" alt="rpict screen" src="https://user-images.githubusercontent.com/941891/99190605-b7945b00-2767-11eb-86c6-84ab7913e94c.png">

#### Cumulated metrics screens (Linky)

This screen displays instantaneous metrics collected from Linky:
- Linky's counter unique id
- "heures creuses" and "heures pleines" indices, used for billing

> <img height="96" alt="linky screen" src="https://user-images.githubusercontent.com/941891/99190602-b6fbc480-2767-11eb-8ca8-b6906ce92347.png">

### Installation

To run the energy-monitor application on a Raspberry Pi:
- install a fresh [Raspberry Pi OS](https://www.raspberrypi.org/software/operating-systems/) on a SD card
- insert the SD card in any Raspberry you own and open a terminal (desktop or ssh)
- configure the system to enable SPI and UART via the [`raspi-config`](https://www.raspberrypi.org/documentation/configuration/raspi-config.md) helper
- intall `libopenjp2-7` and `libtiff5` packages using command `apt-get install -y libopenjp2-7 libtiff5`
- ensure user `pi` belongs to group `spi`, `gpio` and `dialout` using command `usermod -aG spi,gpio,dialout pi`
- install energy-monitor Python package with `pip install {wheel url}`. 
  You can get the wheel url from the [Github Release](https://github.com/ncolomer/energy-monitor/releases) section.
- run the application with the `energy-monitor` binary.
  You can override configuration via environment variable using syntax `env CONFIG1=VALUE1 CONFIG2=VALUE ... energy-monitor`.
  See [Configuration](#Configuration) section below for a full list of overridable configuration.

For advanced users:
- install the Python application in a virtualenv
- use a systemd service to launch the application at startup

### Configuration

You can configure the application by providing the following environment variables:

| Envvar name | Description | Default |
|-|-|-|
| `HMI_SLEEP_SECS` | Duration in seconds before shutting down display | `30` |
| `HMI_MAX_LINE_POWER_WATTS` | Max expected line power in watts | `6900` |
| `HMI_BUTTON_DEBOUNCE_MS` | Push button debounce duration in milliseconds | `200` |
| `RPICT_SERIAL_PORT` | Serial port for RPICT | `/dev/ttyAMA0` |
| `LINKY_SERIAL_PORT` | Serial port for uTeleinfo (Linky) | `/dev/ttyUSB0` |
| `INFLUX_DB_HOST` | InfluxDB host | `localhost` |
| `INFLUX_DB_PORT` | InfluxDB port | `8086` |
| `INFLUX_DB_DATABASE` | InfluxDB database | `metrology` |
| `INFLUX_DB_PREFIX` | Application's measures prefix | `energy` |
| `LOG_LEVEL` | Application log level | `INFO` |


## Hardware

### Parts

The following table presents the partlist needed to build a full module:

| Part | Quantity | Price | Links |
|-|-:|-:|-|
| **Raspberry** Pi Zero WH | 1 | 14.60€ | [buy](https://www.kubii.fr/cartes-raspberry-pi/2076-raspberry-pi-zero-wh-kubii-3272496009394.html) |
| 5V 3A Micro USB Power Supply | 1 | 2.30€ | [buy](https://www.aliexpress.com/item/33000487854.html) |
| **Sandisk** Micro SD card 16GB Class 10 | 1 | 9.90€ | [buy](https://www.kubii.fr/raspberry-pi-microbit/2587-carte-micro-sd-sandisk-16go-classe10-taux-de-transfert-80mb-kubii-619659161613.html) |
| **Waveshare** 2.23" OLED Display 128×32 Pixels SPI/I2C | 1 | 23.00€ | [buy](https://www.amazon.fr/dp/B07XDXZ74V) \| [doc](https://www.waveshare.com/wiki/2.23inch_OLED_HAT) |
| **LeChacal** RPIZ CT3V1 | 1 | 18.30€ | [buy](http://lechacalshop.com/gb/internetofthing/63-rpizct3v1.html) \| [doc](http://lechacal.com/wiki/index.php/RPIZ_CT3V1) |
| **LeChacal** EU AC/AC Adaptor | 1 | 15.00€ | [buy](http://lechacalshop.com/gb/internetofthing/29-acac-adaptor-voltage-sensor-for-rpict-series.html) |
| 3pcs SCT-013-000 Current Transformer | 1 | 11.30€ | [buy](https://www.aliexpress.com/item/32708887594.html) |
| 3pcs 6x6x4.3MM 4PIN G89 Push Button |  1 | 0.70€ | [buy](https://www.aliexpress.com/item/32669948621.html) |
| 200pcs M2.5 Brass Standoffs | 1 | 9.90€ | [buy](https://www.amazon.fr/gp/product/B07QHZNZWD) |
| (optional) **Charles** Micro Teleinfo V2.0 | 1 | 22.90€ | [buy](https://www.tindie.com/products/hallard/micro-teleinfo-v20/) \| [doc](http://hallard.me/category/tinfo/) |
| (optional) 3pcs Micro USB to USB 2.0 adapter | 1 | 5.00€ | [buy](https://www.amazon.fr/gp/product/B00YOX4JU6) |

Total price is roughly 130€, not counting shipping.

### Wiring

In order to achieve wiring between parts, you may need the following tools/parts:

| Part | Price | Link |
|-|-:|-|
| 22AWG Electrical Wire box | 11.80€ | [buy](https://www.aliexpress.com/item/32872439317.html) |
| 620pcs Dupont Connector kit 1 | 4.00€ | [buy](https://www.aliexpress.com/item/4000645340712.html) |
| 310pcs Dupont Connector kit 1 | 4.00€ | [buy](https://www.aliexpress.com/item/4000570942676.html) |
| SN-28B Pin Crimping Tool | 4.00€ | [buy](https://www.aliexpress.com/item/33048867532.html) |

#### RPICT

| RPICT | Raspberry Pi | Physical |
| :--: | :----------: | :------: |
| 1 | 3.3 | 1 |
| 6 | GND | 6 |
| 8 | GPIO14 (UART0_TXD) | 8 |
| 10 | GPIO15 (UART0_RXD) | 10 |

#### OLED display

| OLED | Raspberry Pi | Physical | BCM |
| :--: | :----------: | :------: | :-: |
| VCC | 3.3 | 17 | - |
| GND | GND | 20 | - |
| DIN | MOSI | 19 | 10 |
| CLK | SCLK | 23 | 11 |
| CS  | CE0 | 24 | 8 |
| D/C | GPIO5 | 18 | 24 |
| RES | GPIO6 | 22 | 25 |

#### Push button
GPIO27 pin 13
| Push Button | Raspberry Pi | Physical | BCM |
| :--: | :----------: | :------: | :-: |
| 1 | GPIO27 | 13 | 27 |
| 4 | GND | 14 | - |

### Enclosure

> <img height="256" alt="module cad front" src="https://user-images.githubusercontent.com/941891/99189834-9fbad800-2763-11eb-9791-2a64bb1d46d2.png"> <img height="256" alt="module cad back" src="https://user-images.githubusercontent.com/941891/99189832-9df11480-2763-11eb-82bb-96ae700c2daf.png">

You can find enclosure's STL files in the project's [`enclosure`](https://github.com/ncolomer/energy-monitor/tree/master/enclosure) directory. 
It was designed on [Autodesk Fusion 360](https://www.autodesk.com/products/fusion-360).
It contains 3 clippable parts so that it is easier and faster to print on a 3D printer.
The Raspberry Pi Zero (fixed with the RPICT module with Brass Standoffs) and the oled screen are both clipped to 3D-printed parts.

Assembly only needs a bit of epoxy to hold the push button in place. Don't forget to place the 3D-printed button before glueing the push button!

## Photo gallery

> <img height="256" alt="linky's teleinfo connector" src="https://user-images.githubusercontent.com/941891/99190764-c29bbb00-2768-11eb-9d3c-e3facb948c64.jpg"> <img height="256" alt="utinfo gateway" src="https://user-images.githubusercontent.com/941891/99190773-c7f90580-2768-11eb-8683-aa2838a3d9ae.jpg"> <img height="256" alt="push button cable" src="https://user-images.githubusercontent.com/941891/99190775-c92a3280-2768-11eb-8e07-b209c7906889.jpg"> <img height="256" alt="module components" src="https://user-images.githubusercontent.com/941891/99190778-caf3f600-2768-11eb-8926-0a5af35e1d10.jpg"> <img height="256" alt="module inside once mounted" src="https://user-images.githubusercontent.com/941891/99190922-c0862c00-2769-11eb-8025-d2653013d27d.jpg">
