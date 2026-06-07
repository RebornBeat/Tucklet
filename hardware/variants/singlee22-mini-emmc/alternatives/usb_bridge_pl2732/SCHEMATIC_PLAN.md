# Schematic Plan (PL2732 Alternative) — singlee22-mini-emmc

This is the **PL2732 (USB 3.0)** alternative build path. Same topology as the
standard GL3224 build, but U2 is replaced with the PL2732 QFN-32 bridge.

## Key Differences from Standard (GL3224)
- **U2:** PL2732 (QFN-32, USB 3.0, ~70-100+ MB/s wired) instead of GL3224.
- **eMMC Support:** PL2732 has excellent eMMC HS200 support.
- **Firmware:** May require different firmware initialization sequence. Verify from Prolific datasheet.
- **Pin Mapping:** QFN-32 pinout differs from GL3224. Must verify from PL2732 datasheet.

## Net Changes
- Net topology is identical to GL3224 (same USB 3.0 SS pairs, same SDIO bus).
- Pin numbers on U2 will differ. Update PIN_MAP accordingly.

## All Other Nets
See `../standard/SCHEMATIC_PLAN.md` for the full net list. Only the U2 pin
mappings change; the circuit topology is identical.
