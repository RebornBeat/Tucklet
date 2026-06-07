# Schematic Plan (TMUX1574 Alternative) — singlec5-mini-emmc

This is the **TMUX1574 (Signal Integrity Upgrade)** alternative build path for the SD bus mux.

## Key Differences from Standard (TS3A-class)
- **U6:** TMUX1574 (QFN-24) instead of TS3A-class (QFN-20).
- **Performance:** Higher bandwidth, lower on-resistance (Ron). Better signal integrity for SDIO at higher clock rates.
- **Pin Count:** 24 pins vs 20. Additional pins for control/NC.
- **Use Case:** Recommended if SDIO timing issues are observed with the TS3A-class mux on prototype.

## Net Changes
- **SEL:** Same function (SD_SEL).
- **C_CLK, C_CMD, C_D0..D3:** Same signal names. Pin numbers differ (QFN-24).
- **VCC:** 3V3 supply (same).

## All Other Nets
See `../standard/SCHEMATIC_PLAN.md` for the full net list. Only the U6 pin
mappings change; the circuit topology is identical.
