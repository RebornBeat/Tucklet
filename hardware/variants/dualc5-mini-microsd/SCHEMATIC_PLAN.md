# Schematic Plan (net-by-net) — dualc5-mini-microsd

Build this in KiCad: place the symbols from BOM.csv, then wire the nets
below. Or import `tucklet-dualc5-mini-microsd.net` into Pcbnew to start the
board with a ratsnest directly (logical netlist; reconcile module pins
via PIN_MAP.md).

Variant: **Dual ESP32-C5 (link-aggregated, experimental)**, **microSD (swappable)**, **Mini (Compact)**.

## Power tree
`VBUS` (USB-C) -> U3 charge in; `VBAT` (BT1 <-> U3 BAT <-> U4 CELL <-> U5 VIN);
`+3V3` (U5 VOUT -> U1/U1B, U2, U4, U6, storage, LED, pull-ups).
Common `GND` pour, stitched.

## Storage ownership (the shared-bus trick)
Storage SD_* lines connect to the **common** side of U6. U1 SDIO connects
to U6 A-side; U2 (USB-HS bridge) connects to U6 B-side. `SD_SEL` (driven by
VBUS-present detection in firmware/hardware) selects the owner: plugged in =
bridge owns (fast wired ~70-100+ MB/s); otherwise radio owns (wireless). Never both.

## All nets (signal -> nodes)

| Net | Nodes (ref.pin) |
|---|---|
| `3V3` | U6.VCC, R4.2, R5.2, R6.2, LED1.VDD, R7.2, R8.2, R9.2, R10.2, R11.2, J2.VDD |
| `AGG_CLK` | U1.AGG_CLK, U1B.AGG_CLK |
| `AGG_CS` | U1.AGG_CS, U1B.AGG_CS |
| `AGG_MISO` | U1.AGG_MISO, U1B.AGG_MISO |
| `AGG_MOSI` | U1.AGG_MOSI, U1B.AGG_MOSI |
| `BTN` | R6.1, SW1.1 |
| `CC1` | J1.A5, R1.1 |
| `CC2` | J1.B5, R2.1 |
| `EN` | U1.EN, U1B.EN, U5.3 |
| `GND` | U1.GND, U1B.GND, U2.5, U4.4, U5.2, U6.GND, J1.A1, J1.B1, J1.A12, J1.B12, D1.3, R1.2, R2.2, R3.2, SW1.2, LED1.GND, J2.VSS, BT1.2 |
| `GND_PAD` | U2.32, U3.24 |
| `I2C_SCL` | U1.I2C_SCL, R5.1 |
| `I2C_SDA` | U1.I2C_SDA, R4.1 |
| `PROG` | U3.21, R3.1 |
| `SCL` | U3.5, U4.6 |
| `SDA` | U3.4, U4.7 |
| `SD_CLK` | U2.6, U6.C_CLK, J2.CLK |
| `SD_CMD` | U2.7, U6.C_CMD, R7.1, J2.CMD |
| `SD_D0` | U2.8, U6.C_D0, R8.1, J2.DAT0 |
| `SD_D1` | U2.9, U6.C_D1, R9.1, J2.DAT1 |
| `SD_D2` | U2.10, U6.C_D2, R10.1, J2.DAT2 |
| `SD_D3` | U2.11, U6.C_D3, R11.1, J2.DAT3 |
| `SD_DET` | U2.21, J2.DET |
| `SW` | U3.13, U5.5 |
| `USB_DM` | U2.3, J1.A7, J1.B7, D1.2 |
| `USB_DP` | U2.2, J1.A6, J1.B6, D1.1 |
| `VBUS` | U2.1, U3.1, J1.A4, J1.B4, J1.A9, J1.B9, D1.6 |
| `VDD` | U1.3V3, U1B.3V3, U4.1 |

## Build order in KiCad
power -> U1/U1B + AGG link -> storage + U6 mux -> U2 bridge -> I2C/UI -> charger/gauge. Export the
netlist when done; that's the file to iterate from.
