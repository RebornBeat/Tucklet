# Variant Spec — dualc5-wroom-microsd

**Dual ESP32-C5 (link-aggregated, experimental) + microSD (swappable) (Standard (WROOM))**

## What this board is
A Tucklet build with the dual esp32-c5 (link-aggregated, experimental) radio and
microsd (swappable) storage in a Standard (WROOM) form factor.

## Radio — ESP32-C5
Dual-band Wi-Fi 6 (2.4 + 5 GHz) + BLE. ~15 MB/s wireless at close range.

**Note:** Dual-radio link aggregation is experimental. Validate RF isolation between antennas.
**AGG Link:** SPI (CLK, CS, MOSI, MISO) for higher sustained throughput (~12-15 MB/s).

## Storage — microSD (swappable)
Push-push microSD socket, SDIO 4-bit. Customer supplies/upgrades the card.


## Performance
- Everyday wireless: **~15 MB/s**.
- Bulk wired (GL3224 USB 3.0 bridge): **~70-100+ MB/s**.
- Bulk wired (GL823 USB 2.0 bridge): **~25-35 MB/s**.

## Dimensions
Enclosure envelope: **35 x 28 x 9 mm** (AirTag-class).

## Files in this directory
- `SPEC.md` — Full bill of materials.
- `PIN_MAP.md` — Signal -> MCU pin mapping.
- `SCHEMATIC_PLAN.md` — Net-by-net connection plan.
- `tucklet-dualc5-wroom-microsd.net` — KiCad logical netlist.
- `block_diagram.svg` — System diagram.

## Bring-up checklist
- [ ] Reconcile signal->GPIO from the datasheet (PIN_MAP).
- [ ] Verify GL3224-OEM pin mapping from its datasheet (logical placeholders used).
- [ ] Verify BQ25896 pin mapping from its datasheet (logical placeholders used).
- [ ] Confirm USB-HS bridge part number + handoff mode.
- [ ] Validate SDIO timing.
- [ ] Measure real TX current; size BT1 cell.
- [ ] FCC/CE certification required (radios present).
