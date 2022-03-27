# Display Board

## Max 7219 Info
* [Datasheet](https://datasheets.maximintegrated.com/en/ds/MAX7219-MAX7221.pdf)

## Wiring
* <https://pi4j.com/1.2/pins/model-b-plus.html>
* <http://raspi.tv/2013/8-x-8-led-array-driven-by-max7219-on-the-raspberry-pi-via-python>

| Name  | B+ Pi Pin | Function                    |
|-------|-----------|-----------------------------|
| `VCC` | 4         | 5V                          |
| `GND` | 6         | Ground                      |
| `DIN` | 19        | MOSI (Master out, slave in) |
| `CS`  | 24        | SPI (CE0) Chip Select       |
| `CLK` | 23        | SCLK                        |

## Setup
1. Install [cross](https://github.com/cross-rs/cross) for compiling

## Sunset API Source
(SunRise Sunset)[https://sunrise-sunset.org/]