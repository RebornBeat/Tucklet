# Variant Spec — singlee22-mini-microsd

**Single ESP32-E22 (Wi-Fi 6E) + microSD (swappable) (Mini (Compact))**

## What this board is
A Tucklet build with the single esp32-e22 (wi-fi 6e) radio and
microsd (swappable) storage in a Mini (Compact) form factor.

## Radio — ESP32-E22 (Wi-Fi 6E)
Tri-band Wi-Fi 6E (2.4, 5, 6 GHz) + BLE 5.4.
**High Performance:** ~150 MB/s theoretical wireless throughput.
**Power:** Requires larger battery and robust 3.3V regulation (U5).
**Thermal:** High sustained throughput generates heat; monitor enclosure temps.
**Status:** Roadmap item. Utilizes projected `ESP32-E22-MINI-1` module.

## Storage — microSD (swappable)
Push-push microSD socket, SDIO 4-bit. Customer supplies/upgrades the card.


## Performance
- Everyday wireless: **~150 MB/s**.
- Bulk wired (GL3224 USB 3.0 bridge): **~70-100+ MB/s**.
- Bulk wired (GL823 USB 2.0 bridge): **~25-35 MB/s**.

## Dimensions
Enclosure envelope: **35 x 28 x 9 mm** (AirTag-class).

## Files in this directory
- `BOM.csv` — Full bill of materials.
- `PIN_MAP.md` — Signal -> MCU pin mapping.
- `SCHEMATIC_PLAN.md` — Net-by-net connection plan.
- `tucklet-singlee22-mini-microsd.net` — KiCad logical netlist.
- `block_diagram.svg` — System diagram.

## Bring-up checklist
- [ ] Reconcile signal->GPIO from the datasheet (PIN_MAP).
- [ ] Verify GL3224-OEM pin mapping from its datasheet (logical placeholders used).
- [ ] Verify BQ25896 pin mapping from its datasheet (logical placeholders used).
- [ ] Confirm USB-HS bridge part number + handoff mode.
- [ ] Validate SDIO timing.
- [ ] Measure real TX current; size BT1 cell.
- [ ] FCC/CE certification required (radios present).
