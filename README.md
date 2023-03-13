# energy-monitor

> <img height="256" alt="energy-monitor module" src="https://user-images.githubusercontent.com/941891/99180442-a545fd00-2726-11eb-9eb4-781ce5c0c186.png">
> <img height="256" alt="installed energy-monitor module" src="https://user-images.githubusercontent.com/941891/99187647-32557a00-2758-11eb-86f0-c68d863e7cef.png">
> <img height="256" alt="connections of energy-monitor module" src="https://user-images.githubusercontent.com/941891/99187649-35e90100-2758-11eb-9a7a-da070b3239cc.png">

> “You can't improve what you don't measure”

This project is a DIY module + a Rust application that aims at measuring electrical consumption metrics, display collected values on an OLED display, and send them to an external InfluxDB database for storage. 

The module was designed to fit any standard electrical panel (same form factor as a circuit breaker) and has 90mm width.
It does not collect data directly but rather fetches metrics from [Lechacal](http://lechacal.com/)'s [RPICT](http://lechacal.com/wiki/index.php?title=Raspberrypi_Current_and_Temperature_Sensor_Adaptor) module and Enedis [Linky](https://fr.wikipedia.org/wiki/Linky) electric meter (France national power provider).

I built this project to observe and store my own energy consumption, to eventually improve it. And, well... also because it looked a cool DIY project (it actually was!).

:warning: My electrical installation has three-phase power supply. Even though the project can run on one-phase power supply, it might also need some adaptation. Any contribution welcome!

[![GitHub release](https://img.shields.io/github/v/release/ncolomer/energy-monitor?label=latest%20release&sort=semver&style=for-the-badge)](https://github.com/ncolomer/energy-monitor/releases/latest)

**Table of contents**

* [Application](#application)
  - [Interface](#interface)
  - [Installation](#installation)
  - [Configuration](#configuration)
* [Hardware](#hardware)
  - [Parts](#parts)
  - [Wiring](#wiring)
  - [Enclosure](#enclosure)
* [Photo gallery](#photo-gallery)

## Application

This project includes a Rust application that handles everything, from I/O to display.

### Interface

The user interface is composed of several pages that can be cycled using the push button (like a carousel).

#### Startup screen

> <img height="96" alt="landing screen" src="https://user-images.githubusercontent.com/941891/224688501-87548e33-f3ce-47d1-9e3e-d085a6000a66.png">

This screen displays the project logo, connection statuses and the current application version.
It is shown at application startup and also belongs to the page carousel (last position).

Connection statuses are:
- <img height="16" alt="RPICT" src="https://user-images.githubusercontent.com/941891/147661966-e53ac02a-9856-4179-8a61-28c6abbc21b7.png">
  RPICT status, white square means connected.
- <img height="16" alt="Linky" src="https://user-images.githubusercontent.com/941891/147662128-ec6107df-cc06-4576-aa8f-3746874ef76e.png">
  Linky status, white square means connected.
- <img height="16" alt="InfluxDB" src="https://user-images.githubusercontent.com/941891/147662201-dce46e58-cf2b-474e-b7a1-f6374338cd35.png">
  InfluxDB status, white square means connected.

#### Instantaneous metrics screen (RPICT)

> <img height="96" alt="rpict screen" src="https://user-images.githubusercontent.com/941891/224688506-7cb726e1-43ce-4f90-ba1e-33d954900f29.png">

This screen displays instantaneous metrics measured from the RPICT:
- **lines apparent power** value with a gauge that shows the max seen value since boot
- sum of consumed **lines power**
- mean of lines **RMS voltage**

It is the first displayed screen when waking up from sleep.

#### Cumulated metrics screens (Linky)

> <img height="96" alt="linky screen" src="https://user-images.githubusercontent.com/941891/224688505-4113e42a-946a-4c44-91ea-93ce31b0b1bb.png">

This screen displays instantaneous metrics collected from the Linky:
- Linky's counter **unique id**
- "heures creuses" and "heures pleines" **indices**, used for billing

### Installation

To run the energy-monitor application on a Raspberry Pi Zero W:
- install a fresh [Raspberry Pi OS](https://www.raspberrypi.org/software/operating-systems/) on a SD card
- insert the SD card and open a terminal (desktop or ssh)
- configure the system to enable SPI and UART via the [`raspi-config`](https://www.raspberrypi.org/documentation/configuration/raspi-config.md) helper
- ensure user `pi` belongs to group `spi`, `gpio` and `dialout` using command `usermod -aG spi,gpio,dialout pi`
- download energy-monitor binary from the [Github Release](https://github.com/ncolomer/energy-monitor/releases) section
- run the application with the `energy-monitor` binary.
  You can override configuration using YAML config file or environment variables.
  See [Configuration](#configuration) section below for a full list of overridable settings.

```
$ energy-monitor -h
A tool to measure, display and store electrical consumption metrics.

Usage: energy-monitor [OPTIONS]

Options:
  -c, --config <FILE>  Sets a custom YAML config file
  -h, --help           Print help
  -V, --version        Print version
```

For advanced users:
- use a systemd service to launch the application at system startup

### Configuration

You can configure the application either by providing a YAML config file (see `-c --config <FILE>` binary arg) or using environment variables:

| YAML path                  | Environment variable             | Description                                      | Default        |
|----------------------------|----------------------------------|--------------------------------------------------|----------------|
| `log_level`                | `APP__LOG_LEVEL`                 | Application log level                            | `INFO`         |
| `hmi.sleep_timeout_secs`   | `APP__HMI__SLEEP_TIMEOUT_SECS`   | Duration in seconds before shutting down display | `30`           |
| `hmi.max_line_power_watts` | `APP__HMI__MAX_LINE_POWER_WATTS` | Max expected line power in watts                 | `6900`         |
| `hmi.button_debounce_ms`   | `APP__HMI__BUTTON_DEBOUNCE_MS`   | Push button debounce duration in milliseconds    | `100`          |
| `hmi.button_bcm_pin`       | `APP__HMI__BUTTON_BCM_PIN`       | Push button BCM pin number                       | `27`           |
| `serial.rpict`             | `APP__SERIAL__RPICT`             | Serial port for RPICT                            | `/dev/ttyAMA0` |
| `serial.linky`             | `APP__SERIAL__LINKY`             | Serial port for uTeleinfo (Linky)                | `/dev/ttyUSB0` |
| `influxdb.host`            | `APP__INFLUXDB__HOST`            | InfluxDB host                                    | `localhost`    |
| `influxdb.port`            | `APP__INFLUXDB__PORT`            | InfluxDB port                                    | `8086`         |
| `influxdb.database`        | `APP__INFLUXDB__DATABASE`        | InfluxDB database                                | `metrology`    |
| `influxdb.prefix`          | `APP__INFLUXDB__PREFIX`          | Application's measures prefix                    | `energy`       |

## Hardware

The module is composed of several parts listed in the [Parts](#parts) section.
Electronics parts (push button, OLED display and RPICT) are wired to the Raspberry Pi, see [Wiring](#wiring) section.
RPICT is tied to the Raspberry Pi module via 29.7mm Brass Standoffs, they are both tied to enclosure's part 1 via clips.
The OLED display is tied to enclosure's part 3 the same way.

### Parts

The following table presents the partlist needed to build a full module:

| Part                                                   | Quantity | Price  | Links                                                                                                                                      |
|--------------------------------------------------------|----------|--------|--------------------------------------------------------------------------------------------------------------------------------------------|
| **Raspberry** Pi Zero WH                               | 1        | 14.60€ | [buy](https://www.kubii.fr/cartes-raspberry-pi/2076-raspberry-pi-zero-wh-kubii-3272496009394.html)                                         |
| 5V 3A Micro USB Power Supply                           | 1        | 2.30€  | [buy](https://www.aliexpress.com/item/33000487854.html)                                                                                    |
| **Sandisk** Micro SD card 16GB Class 10                | 1        | 9.90€  | [buy](https://www.kubii.fr/raspberry-pi-microbit/2587-carte-micro-sd-sandisk-16go-classe10-taux-de-transfert-80mb-kubii-619659161613.html) |
| **Waveshare** 2.23" OLED Display 128×32 Pixels SPI/I2C | 1        | 23.00€ | [buy](https://www.amazon.fr/dp/B07XDXZ74V) / [doc](https://www.waveshare.com/wiki/2.23inch_OLED_HAT)                                       |
| **LeChacal** RPIZ CT3V1                                | 1        | 18.30€ | [buy](http://lechacalshop.com/gb/internetofthing/63-rpizct3v1.html) / [doc](http://lechacal.com/wiki/index.php/RPIZ_CT3V1)                 |
| **LeChacal** EU AC/AC Adaptor                          | 1        | 15.00€ | [buy](http://lechacalshop.com/gb/internetofthing/29-acac-adaptor-voltage-sensor-for-rpict-series.html)                                     |
| 3pcs SCT-013-000 Current Transformer                   | 1        | 11.30€ | [buy](https://www.aliexpress.com/item/32708887594.html)                                                                                    |
| 3pcs 6x6x4.3MM 4PIN G89 Push Button                    | 1        | 0.70€  | [buy](https://www.aliexpress.com/item/32669948621.html)                                                                                    |
| 200pcs M2.5 Brass Standoffs                            | 1        | 9.90€  | [buy](https://www.amazon.fr/gp/product/B07QHZNZWD)                                                                                         |
| (optional) **Charles** Micro Teleinfo V2.0             | 1        | 22.90€ | [buy](https://www.tindie.com/products/hallard/micro-teleinfo-v20/) / [doc](http://hallard.me/category/tinfo/)                              |
| (optional) 3pcs Micro USB to USB 2.0 adapter           | 1        | 5.00€  | [buy](https://www.amazon.fr/gp/product/B00YOX4JU6)                                                                                         |

Total price is roughly 130€, not counting shipping.

### Wiring

The following tables uses the following Raspberry Pi pinout as reference:

> <img height="128" alt="raspberry pi pinout reference" src="https://user-images.githubusercontent.com/941891/99191467-2b853200-276d-11eb-9982-401c0d7a61ec.png">

In order to achieve wiring between parts, you may need the following tools/parts:

| Part                          | Price  | Link                                                      |
|-------------------------------|--------|-----------------------------------------------------------|
| 22AWG Electrical Wire box     | 11.80€ | [buy](https://www.aliexpress.com/item/32872439317.html)   |
| 620pcs Dupont Connector kit 1 | 4.00€  | [buy](https://www.aliexpress.com/item/4000645340712.html) |
| 310pcs Dupont Connector kit 1 | 4.00€  | [buy](https://www.aliexpress.com/item/4000570942676.html) |
| SN-28B Pin Crimping Tool      | 4.00€  | [buy](https://www.aliexpress.com/item/33048867532.html)   |

#### RPICT

| RPICT | Raspberry Pi       | Physical |
|-------|--------------------|----------|
| 1     | 3.3                | 1        |
| 6     | GND                | 6        |
| 8     | GPIO14 (UART0_TXD) | 8        |
| 10    | GPIO15 (UART0_RXD) | 10       |

#### OLED display

| OLED | Raspberry Pi | Physical | BCM |
|------|--------------|----------|-----|
| VCC  | 3.3          | 17       | -   |
| GND  | GND          | 20       | -   |
| DIN  | MOSI         | 19       | 10  |
| CLK  | SCLK         | 23       | 11  |
| CS   | CE0          | 24       | 8   |
| D/C  | GPIO5        | 18       | 24  |
| RES  | GPIO6        | 22       | 25  |

#### Push button

| Push Button | Raspberry Pi | Physical | BCM |
|-------------|--------------|----------|-----|
| 1           | GPIO27       | 13       | 27  |
| 4           | GND          | 14       | -   |

### Enclosure

> <img height="192" alt="print plate" src="https://user-images.githubusercontent.com/941891/99191792-51133b00-276f-11eb-9765-e226e3d1bb20.png">
> <img height="192" alt="module cad front" src="https://user-images.githubusercontent.com/941891/99189834-9fbad800-2763-11eb-9791-2a64bb1d46d2.png">
> <img height="192" alt="module cad back" src="https://user-images.githubusercontent.com/941891/99189832-9df11480-2763-11eb-82bb-96ae700c2daf.png">

The enclosure contains 3 clippable parts so that it is easier and faster to print on a 3D printer. 
Assembly only needs a bit of epoxy to hold the push button in place. Don't forget to place the 3D-printed button before glueing the push button!
The enclosure was designed using [Autodesk Fusion 360](https://www.autodesk.com/products/fusion-360).

You can find `emonitor-part*.stl` STL files in the project's [`enclosure`](https://github.com/ncolomer/energy-monitor/tree/master/enclosure) directory.

I printed mine in 6 hours, using [PrusaSlicer](https://www.prusa3d.com/prusaslicer/) as slicer, and with the following parameters:
- 0.20mm SPEED profile
- 20% infill
- support on build plate only
- white 1.75mm PLA filament

Notes:
- don't forget to print the button in your favorite color, see `button.stl` STL file
- I also designed an enclosure for utinfo, see `uteleinfo-part*.stl` STL files

## Photo gallery

> <img height="256" alt="linky's teleinfo connector" src="https://user-images.githubusercontent.com/941891/99190764-c29bbb00-2768-11eb-9d3c-e3facb948c64.jpg">
> <img height="256" alt="utinfo gateway" src="https://user-images.githubusercontent.com/941891/99190773-c7f90580-2768-11eb-8683-aa2838a3d9ae.jpg">
> <img height="256" alt="push button cable" src="https://user-images.githubusercontent.com/941891/99190775-c92a3280-2768-11eb-8e07-b209c7906889.jpg">
> <img height="256" alt="module components" src="https://user-images.githubusercontent.com/941891/99190778-caf3f600-2768-11eb-8926-0a5af35e1d10.jpg">
> <img height="256" alt="module inside once mounted" src="https://user-images.githubusercontent.com/941891/99190922-c0862c00-2769-11eb-8025-d2653013d27d.jpg">
