# Schematic Plan (net-by-net) — duale22-mini-emmc

Build this in KiCad: place the symbols from BOM.csv, then wire the nets
below. Or import `tucklet-duale22-mini-emmc.net` into Pcbnew to start the
board with a ratsnest directly (logical netlist; reconcile module pins
via PIN_MAP.md).

Variant: **Dual ESP32-E22 (High-Performance)**, **eMMC (sealed)**, **Mini (Compact)**.

## Power tree
`VBUS` (USB-C) -> U3 charge in; `VBAT` (BT1 <-> U3 VBAT <-> U4 CELL);
`VSYS` (U3 SYS -> U5 VIN); `+3V3` (U5 SW(LX) -> U1/U1B, U2, U4, U6, storage, LED, pull-ups).
Common `GND` pour, stitched.

## Storage ownership (the shared-bus trick via TXS02612)
Storage SD_* lines connect to the **A-side** of U6. U1 SDIO connects
to U6 **B0-side** (MCU_SD_* nets); U2 (USB-HS bridge) connects to U6 **B1-side** (BRG_SD_* nets).
`SD_SEL` (driven by VBUS-present detection in firmware/hardware) selects the owner: plugged in =
bridge owns (B1 active, fast wired ~70-100+ MB/s); otherwise radio owns (B0 active, wireless). Never both.

## All nets (signal -> nodes)

| Net | Nodes (ref.pin) |
|---|---|
| `3V3` | U1.2, U1B.2, U4.3, U5.3, U6.5, U6.17, U6.21, R4.2, R5.2, R6.2, LED1.4, R7.2, R8.2, R9.2, R10.2, R11.2, XU7.VCC, XU7.VCCQ |
| `AGG_CLK` | U1.26, U1B.26 |
| `AGG_CS` | U1.27, U1B.27 |
| `AGG_MISO` | U1.15, U1B.15 |
| `AGG_MOSI` | U1.18, U1B.18 |
| `BRG_SD_CLK` | U2.20, U6.13 |
| `BRG_SD_CMD` | U2.21, U6.12 |
| `BRG_SD_D0` | U2.19, U6.14 |
| `BRG_SD_D1` | U2.18, U6.15 |
| `BRG_SD_D2` | U2.23, U6.8 |
| `BRG_SD_D3` | U2.22, U6.10 |
| `BTN` | U1.6, R6.1, SW1.1 |
| `CC1` | J1.A5, R1.1 |
| `CC2` | J1.B5, R2.1 |
| `EN` | U1.3, U1B.3, U5.1 |
| `GAUGE_ALRT` | U1.8, U4.5 |
| `GND` | U1.1, U1.28, U1.29, U1B.1, U1B.28, U1B.29, U2.27, U4.4, U4.9, U5.2, U6.2, U6.11, U6.25, J1.1, J1.2, J1.3, J1.4, J1.A1B12, J1.B1A12, D1.3, R1.2, R2.2, R3.2, SW1.2, LED1.2, XU7.VSS, XU7.VSSQ, BT1.2 |
| `GND_PAD` | U2.33, U3.25 |
| `I2C_SCL` | U1.16, U4.7, R5.1 |
| `I2C_SDA` | U1.17, U4.8, R4.1 |
| `ILIM` | U3.10, R3.1 |
| `LED_DIN` | U1.4, LED1.3 |
| `MCU_SD_CLK` | U1.14, U6.19 |
| `MCU_SD_CMD` | U1.13, U6.20 |
| `MCU_SD_D0` | U1.12, U6.18 |
| `MCU_SD_D1` | U1.11, U6.16 |
| `MCU_SD_D2` | U1.10, U6.23 |
| `MCU_SD_D3` | U1.9, U6.22 |
| `SD_CLK` | U6.9, XU7.CLK |
| `SD_CMD` | U6.4, R7.1, XU7.CMD |
| `SD_D0` | U6.6, R8.1, XU7.DAT0 |
| `SD_D1` | U6.7, R9.1, XU7.DAT1 |
| `SD_D2` | U6.1, R10.1, XU7.DAT2 |
| `SD_D3` | U6.3, R11.1, XU7.DAT3 |
| `SD_SEL` | U1.21, U6.24 |
| `USB_DM` | U2.31, J1.A7, J1.B7, D1.2 |
| `USB_DP` | U2.32, J1.A6, J1.B6, D1.1 |
| `VBAT` | U3.13, U4.2, BT1.1 |
| `VBUS` | U2.15, U3.1, J1.A4B9, J1.B4A9, D1.6 |
| `VSYS` | U3.15, U5.4 |

## Build order in KiCad
power -> U1/U1B + AGG link -> storage + U6 mux -> U2 bridge -> I2C/UI -> charger/gauge. Export the
netlist when done; that's the file to iterate from.
