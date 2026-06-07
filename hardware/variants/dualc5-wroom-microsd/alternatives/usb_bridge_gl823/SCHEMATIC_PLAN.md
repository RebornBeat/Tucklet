# Schematic Plan (GL823 Alternative) — dualc5-wroom-microsd

This is the **GL823 (USB 2.0)** alternative build path. Same topology as the
standard GL3224 build, but U2 is replaced with the GL823 QFN-24 bridge.

## Key Differences from Standard (GL3224)
- **U2:** GL823 (QFN-24, USB 2.0, ~25-35 MB/s wired) instead of GL3224 (QFN-32, USB 3.0, ~70-100+ MB/s)
- **Pin Count:** 24 pins vs 32 pins. GL823 lacks USB 3.0 SuperSpeed differential pairs.
- **Performance:** Wired speed is limited to USB 2.0 High-Speed (~25-35 MB/s).
- **Footprint:** Smaller (4x4 mm QFN-24 vs 5x5 mm QFN-32).

## Net Changes
- **USB_SS_TX_P, USB_SS_TX_N, USB_SS_RX_P, USB_SS_RX_N:** Not present. USB 3.0 lanes are absent.
- All other nets (VBUS, USB_DP, USB_DM, SD_CLK, SD_CMD, SD_D0..D3, SD_SEL) remain identical.
- **VCC_33 -> VDD33:** GL823 uses VDD33 for 3.3V supply. Functionally equivalent to GL3224's VCC_33.
- **RREF:** GL823 may not require an external reference resistor. Verify from datasheet.

## All Other Nets
See `../standard/SCHEMATIC_PLAN.md` for the full net list. Only the U2 pin
mappings change; the circuit topology is identical.
