# energy-monitor

<img width="512" alt="energy-monitor module" src="https://user-images.githubusercontent.com/941891/99180442-a545fd00-2726-11eb-9eb4-781ce5c0c186.png">

> You can't improve what you don't measure

This project is a DIY module + a Python application that aim at measuring electrical consumption metrics, display collected values on an OLED display, and send them to an external InfluxDB database for historization. 

The module was built to fit any standard electrical panel (same form factor as a circuit breaker). 
It does not collect data directly but uses data from [Lechacal](http://lechacal.com/)'s [RPICT](http://lechacal.com/wiki/index.php?title=Raspberrypi_Current_and_Temperature_Sensor_Adaptor) and [Linky](https://fr.wikipedia.org/wiki/Linky) (french power supply counter).

I built this project to observe and store my own energy consumption, to eventually improve them. And also because it looked —and was— a cool DIY project.

![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/ncolomer/energy-monitor?label=latest%20release&sort=semver&style=for-the-badge)

## Application

### Interface

#### Startup screen

The screen and the current application version are displayed at application startup.
It also belongs to the page carousel, in the last position.

> <img width="512" alt="landing" src="https://user-images.githubusercontent.com/941891/99180029-04097780-2723-11eb-8937-fbdda2956b72.png">

#### Instantaneous metrics screen (RPICT)

This screen displays instantaneous metrics measured from RPICT:
- lines apparent power value, also displayed on a bar which shows the max value seen since boot
- sum of consumed lines power
- mean of lines rms voltage

It is the first displayed when waking up from sleep.

> <img width="512" alt="rpict" src="https://user-images.githubusercontent.com/941891/99180031-053aa480-2723-11eb-8e47-bc9ecee9510e.png">

#### Cumulated metrics screens (Linky)

This screen displays instantaneous metrics collected from Linky:
- the counter unique id
- "heures creuses" and "heures pleines" indices, used for billing

> <img width="512" alt="linky" src="https://user-images.githubusercontent.com/941891/99180030-04a20e00-2723-11eb-9fb6-2d6153d8b24a.png">

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

### Run as a service

## Hardware

### Parts

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

| OLED | Raspberry Pi | Physical | BCM |
| :--: | :----------: | :------: | :-: |
| VCC | 3.3 | 17 | - |
| GND | GND | 20 | - |
| DIN | MOSI | 19 | 10 |
| CLK | SCLK | 23 | 11 |
| CS  | CE0 | 24 | 8 |
| D/C | GPIO5 | 18 | 24 |
| RES | GPIO6 | 22 | 25 |

| Part | Price | Link |
|-|-:|-|
| 22AWG Electrical Wire box | 11.80€ | [buy](https://www.aliexpress.com/item/32872439317.html) |
| 620pcs Dupont Connector kit 1 | 4.00€ | [buy](https://www.aliexpress.com/item/4000645340712.html) |
| 310pcs Dupont Connector kit 1 | 4.00€ | [buy](https://www.aliexpress.com/item/4000570942676.html) |
| SN-28B Pin Crimping Tool | 4.00€ | [buy](https://www.aliexpress.com/item/33048867532.html) |



### Enclosure
