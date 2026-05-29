# Variant Spec — singlec5-microsd

**Single ESP32-C5 + microSD (swappable)**

## What this board is
A Tucklet build with the single esp32-c5 radio and
microsd (swappable) storage. It shares the entire common
design (USB-C charge, MCP73831 charger, MAX17048 fuel gauge, 3V3 buck, USB-HS
storage bridge, SD mux, button, RGB LED) with the other variants; only the
storage block differs.

## Connectivity (all variants support the full layered stack)
- **BLE** — discovery, auth, battery %, free space, wake. Always on (low duty).
- **Wi-Fi SoftAP** — universal wireless data path (works on every phone today).
- **Wi-Fi Aware (NAN)** — seamless wireless path where the firmware feature is
  enabled and the device is certified (see docs/FINAL_REVIEW.md). Same radio, so
  same throughput as SoftAP; it improves *seamlessness*, not speed.
- **USB-HS wired bridge** — ~20-40 MB/s when plugged in (U2 owns storage).

Transport support is a firmware feature flag, not a board change — every board
can run SoftAP-only or SoftAP+Aware. See docs/VARIANT_MATRIX.md.

## Radio — Single ESP32-C5
One dual-band Wi-Fi 6 (2.4 + 5 GHz) + BLE module. ~9 MB/s wireless at close
range (the charm sits on the phone, so the link is centimeters long and sustains
top modulation). BLE is the control/auth plane; Wi-Fi is the data plane.

## Storage — microSD (swappable)
Push-push microSD socket, SDIO 4-bit. Customer supplies/upgrades the card; you
don't pay for the flash. A dead card is a $5 swap, not a dead device. Slightly
larger enclosure than eMMC (socket + insertion clearance), internal behind a
small door so the device still looks sealed day-to-day.

## Performance
- Everyday wireless (close range): **~9 MB/s**.
- Bulk wired (USB-HS bridge): **~20-40 MB/s**.
- Do NOT market the radio's own USB path (~0.5 MB/s, unused; see TRANSFER_PERFORMANCE).

## Dimensions
Enclosure envelope: **35 x 28 x 9 mm** (AirTag-class). The ESP32-C5 module
is the footprint driver; the battery is the thickness driver. Adding the USB-HS
bridge () does not change
the envelope class.

## Files in this directory
- `BOM.csv` — full bill of materials (electronics + PCB + enclosure).
- `PIN_MAP.md` — signal -> C5 pin reconciliation (datasheet step flagged).
- `SCHEMATIC_PLAN.md` — net-by-net connection plan.
- `tucklet-singlec5-microsd.net` — KiCad-importable netlist (Pcbnew > Import Netlist).
- `block_diagram.svg` — this variant's block diagram.

## Bring-up checklist (settle on real silicon)
- [ ] Reconcile signal->GPIO from the ESP32-C5-WROOM-1 datasheet (PIN_MAP).
- [ ] Confirm the USB-HS bridge part number + its SD shared-bus/handoff mode.
- [ ] Validate SDIO timing to the microSD card.
- [ ] Measure real 5 GHz transfer current; size BT1 cell.

- [ ] FCC/CE (radios present); Wi-Fi Alliance cert only if shipping Aware on iOS.
