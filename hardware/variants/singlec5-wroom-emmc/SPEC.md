# Variant Spec — singlec5-wroom-emmc

**Single ESP32-C5 + eMMC (sealed) (Standard (WROOM))**

## What this board is
A Tucklet build with the single esp32-c5 radio and
emmc (sealed) storage in a Standard (WROOM) form factor.

## Radio — ESP32-C5
Dual-band Wi-Fi 6 (2.4 + 5 GHz) + BLE. ~9 MB/s wireless at close range.

## Storage — eMMC (sealed, per-SKU capacity)
153-ball TFBGA eMMC 5.1, 11.5 x 13.0 x 1.2 mm. Capacity is fixed at manufacture.

| Capacity | eMMC est. cost | Notes |
|---|---|---|
| 8 GB | ~$2.50 | sealed |
| 16 GB | ~$3.50 | sealed |
| 32 GB | ~$5.00 | sealed |
| 64 GB | ~$8.00 | sealed |
| 128 GB | ~$13.00 | sealed |
| 256 GB | ~$22.00 | sealed |


## Performance
- Everyday wireless: **~9 MB/s**.
- Bulk wired (USB-HS bridge): **~20-40 MB/s**.

## Dimensions
Enclosure envelope: **32 x 24 x 8 mm** (AirTag-class).

## Files in this directory
- `BOM.csv` — Full bill of materials.
- `PIN_MAP.md` — Signal -> MCU pin mapping.
- `SCHEMATIC_PLAN.md` — Net-by-net connection plan.
- `tucklet-singlec5-wroom-emmc.net` — KiCad logical netlist.
- `block_diagram.svg` — System diagram.

## Bring-up checklist
- [ ] Reconcile signal->GPIO from the datasheet (PIN_MAP).
- [ ] Confirm USB-HS bridge part number + handoff mode.
- [ ] Validate SDIO timing.
- [ ] Measure real TX current; size BT1 cell.
- [ ] FCC/CE certification required (radios present).
