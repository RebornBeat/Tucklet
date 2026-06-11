# Pin Map — singlec5-mini-microsd (ESP32-C5)

> **Read this first.** The standard baseline GPIO numbers are mapped below based
> on the finalized ESP32-C5-WROOM-1-N8R8 symbol pin extraction. Keep the **strapping pins clean**.

## Fixed-function (must use the dedicated pins)
| Signal | Function | Pin | Notes |
|---|---|---|---|
| USB_DP / USB_DM | native USB (flashing/JTAG; NOT the data path) | dedicated USB pins | route to test pads; storage data goes through the USB-HS bridge instead |
| 3V3 / GND / EN | power + enable | Pin 2 / Pins 1,28,29 / Pin 3 | EN via RC reset network |

## Storage bus — SDIO 4-bit (segmented via U6 TXS02612 mux)
| Signal | Function | Pin | Notes |
|---|---|---|---|
| MCU_SD_CLK | SDIO clock to Mux B0 | IO14 (Pin 14) | to U6 CLKB0 |
| MCU_SD_CMD | SDIO command to Mux B0 | IO13 (Pin 13) | 10k pull-up on A-side |
| MCU_SD_D0..D3 | SDIO data 0-3 to Mux B0 | IO10..IO7 (Pins 12..9) | 10k pull-ups on A-side; D3=CS in 1-bit/SPI fallback |
| SD_SEL | storage ownership select | IO23 (Pin 21) | drives U6 SEL; high=bridge owns (USB plugged), low=radio owns |
| SD_DET | card-detect | IO24 (Pin 23) | microSD insertion detect |

## I2C (fuel gauge U4 + charger U3)
| Signal | Function | Pin | Notes |
|---|---|---|---|
| I2C_SDA | I2C data | IO4 (Pin 17) | 4.7k pull-up; shared with BQ25896 SDA |
| I2C_SCL | I2C clock | IO5 (Pin 16) | 4.7k pull-up; shared with BQ25896 SCL |
| GAUGE_ALRT | low-battery interrupt | IO6 (Pin 8) | from MAX17048 ALRT (open-drain) |
| CHG_INT | charger interrupt | IO25 (Pin 26) | from BQ25896 INT (open-drain). *Mapped to AGG_CLK on Dual variants.* |

## UI + charger status
| Signal | Function | Pin | Notes |
|---|---|---|---|
| BTN | tactile button | IO0 (Pin 6) | 10k pull-up + firmware debounce; press patterns = pair / factory-reset |
| LED_DIN | WS2812 data | IO2 (Pin 4) | single addressable RGB |
| CHG_STAT | charger status | IO3 (Pin 5) | from BQ25896 STAT (open-drain) |

## Routing note
Wi-Fi 5 GHz TX bursts are the current peak. Wide copper on VBUS/VBAT/VSYS/3V3, bulk +
decoupling close to U1 (and U1B on dual), and size U5 for the peak, not the
average — this prevents mid-transfer brownout resets.
