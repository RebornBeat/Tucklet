# Variant Spec — singlec5-wroom-microsd

**Single ESP32-C5 + microSD (swappable) (Standard (WROOM))**

## What this board is
A Tucklet build with the single esp32-c5 radio and
microsd (swappable) storage in a Standard (WROOM) form factor.

## Radio — ESP32-C5
Dual-band Wi-Fi 6 (2.4 + 5 GHz) + BLE. ~9 MB/s wireless at close range.

## Storage — microSD (swappable)
Push-push microSD socket, SDIO 4-bit. Customer supplies/upgrades the card.


## Performance
- Everyday wireless: **~9 MB/s**.
- Bulk wired (USB-HS bridge): **~20-40 MB/s**.

## Dimensions
Enclosure envelope: **35 x 28 x 9 mm** (AirTag-class).

## Files in this directory
- `BOM.csv` — Full bill of materials.
- `PIN_MAP.md` — Signal -> MCU pin mapping.
- `SCHEMATIC_PLAN.md` — Net-by-net connection plan.
- `tucklet-singlec5-wroom-microsd.net` — KiCad logical netlist.
- `block_diagram.svg` — System diagram.

## Bring-up checklist
- [ ] Reconcile signal->GPIO from the datasheet (PIN_MAP).
- [ ] Confirm USB-HS bridge part number + handoff mode.
- [ ] Validate SDIO timing.
- [ ] Measure real TX current; size BT1 cell.
- [ ] FCC/CE certification required (radios present).
