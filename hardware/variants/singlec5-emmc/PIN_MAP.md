# Pin Map — singlec5-emmc (ESP32-C5-WROOM-1)

> **Read this first.** GPIO numbers are NOT hard-coded here because the
> ESP32-C5-WROOM-1 pin table must be taken from the **current datasheet**, and
> the C5 map differs from the S3. Assign each signal below to a free C5 GPIO,
> keeping the **strapping pins clean** (do not load boot-strap pins with
> peripherals that disturb boot). This one reconciliation step turns the
> logical netlist into a routable schematic.

## Fixed-function (must use the dedicated pins)
| Signal | Function | C5 pin | Notes |
|---|---|---|---|
| USB_DP / USB_DM | native USB (flashing/JTAG; NOT the data path) | dedicated USB pins | route to test pads; storage data goes through the USB-HS bridge instead |
| 3V3 / GND / EN | power + enable | dedicated | EN via RC reset network |

## Storage bus — SDIO 4-bit (shared via U6 mux)
| Signal | Function | C5 pin | Notes |
|---|---|---|---|
| SD_CLK | SDIO clock | free GPIO | to U6 A-side |
| SD_CMD | SDIO command | free GPIO | 10k pull-up |
| SD_D0..D3 | SDIO data 0-3 | free GPIOs (contiguous preferred) | 10k pull-ups each; D3=CS in 1-bit/SPI fallback |
| SD_SEL | storage ownership select | free GPIO | drives U6; high=bridge owns (USB plugged), low=radio owns |
| EMMC_RST | eMMC hardware reset | free GPIO | drive per JEDEC reset timing |
| (DAT4..DAT7) | optional 8-bit eMMC | free GPIOs | only if bridge + layout support 8-bit; raises wired throughput |


## I2C (fuel gauge U4)
| Signal | Function | C5 pin | Notes |
|---|---|---|---|
| I2C_SDA | I2C data | free GPIO | 4.7k pull-up |
| I2C_SCL | I2C clock | free GPIO | 4.7k pull-up |
| GAUGE_ALRT | low-battery interrupt | free GPIO | from MAX17048 ALRT (open-drain) |

## UI + charger status
| Signal | Function | C5 pin | Notes |
|---|---|---|---|
| BTN | tactile button | free GPIO | 10k pull-up + firmware debounce; press patterns = pair / factory-reset |
| LED_DIN | WS2812 data | free GPIO | single addressable RGB |
| CHG_STAT | charger status | free GPIO | from MCP73831 STAT (open-drain) |

## Routing note
Wi-Fi 5 GHz TX bursts are the current peak. Wide copper on VBUS/VBAT/3V3, bulk +
decoupling close to U1 (and U1B on dual), and size U5 for the peak, not the
average — this prevents mid-transfer brownout resets.
