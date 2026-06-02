# Schematic Plan (net-by-net) — duale22-wroom-emmc

Build this in KiCad: place the symbols from BOM.csv, then wire the nets
below. Or import `tucklet-duale22-wroom-emmc.net` into Pcbnew to start the
board with a ratsnest directly (logical netlist; reconcile module pins
via PIN_MAP.md).

Variant: **Dual ESP32-E22 (High-Performance)**, **eMMC (sealed)**, **Standard (WROOM)**.

## Power tree
`VBUS` (USB-C) -> U3 charge in; `VBAT` (BT1 <-> U3 BAT <-> U4 CELL <-> U5 VIN);
`+3V3` (U5 VOUT -> U1/U1B, U2, U4, U6, storage, LED, pull-ups).
Common `GND` pour, stitched.

## Storage ownership (the shared-bus trick)
Storage SD_* lines connect to the **common** side of U6. U1 SDIO connects
to U6 A-side; U2 (USB-HS bridge) connects to U6 B-side. `SD_SEL` (driven by
VBUS-present detection in firmware/hardware) selects the owner: plugged in =
bridge owns (fast wired ~20-40 MB/s); otherwise radio owns (wireless). Never both.

## All nets (signal -> nodes)

| Net | Nodes (ref.pin) |
|---|---|
| `3V3` | U2.4, U6.VCC, R4.2, R5.2, R6.2, LED1.VDD, R7.2, R8.2, R9.2, R10.2, R11.2, XU7.VCC, XU7.VCCQ |
| `AGG_A` | U1.AGG_TX, U1B.AGG_RX |
| `AGG_B` | U1.AGG_RX, U1B.AGG_TX |
| `BTN` | R6.1, SW1.1 |
| `CC1` | J1.A5, R1.1 |
| `CC2` | J1.B5, R2.1 |
| `EN` | U1.EN, U1B.EN, U5.3 |
| `GND` | U1.GND, U1B.GND, U2.5, U4.4, U5.2, U6.GND, J1.A1, J1.B1, J1.A12, J1.B12, D1.3, R1.2, R2.2, R3.2, SW1.2, LED1.GND, XU7.VSS, XU7.VSSQ, BT1.2 |
| `I2C_SCL` | U1.I2C_SCL, R5.1 |
| `I2C_SDA` | U1.I2C_SDA, R4.1 |
| `PROG` | U3.5, R3.1 |
| `SD_CLK` | U2.6, U6.C_CLK, XU7.CLK |
| `SD_CMD` | U2.7, U6.C_CMD, R7.1, XU7.CMD |
| `SD_D0` | U2.8, U6.C_D0, R8.1, XU7.DAT0 |
| `SD_D1` | U2.9, U6.C_D1, R9.1, XU7.DAT1 |
| `SD_D2` | U2.10, U6.C_D2, R10.1, XU7.DAT2 |
| `SD_D3` | U2.11, U6.C_D3, R11.1, XU7.DAT3 |
| `USB_DM` | U2.3, J1.A7, J1.B7, D1.2 |
| `USB_DP` | U2.2, J1.A6, J1.B6, D1.1 |
| `VBAT` | U3.3, BT1.1 |
| `VBUS` | U2.1, J1.A4, J1.B4, J1.A9, J1.B9, D1.6 |
| `VDD` | U1.3V3, U1B.3V3, U3.4, U4.1 |

## Build order in KiCad
power -> U1/U1B + AGG link -> storage + U6 mux -> U2 bridge -> I2C/UI -> charger/gauge. Export the
netlist when done; that's the file to iterate from.
