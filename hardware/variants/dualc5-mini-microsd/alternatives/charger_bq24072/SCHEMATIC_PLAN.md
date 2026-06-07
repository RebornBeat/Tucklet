# Schematic Plan (BQ24072 Alternative) — dualc5-mini-microsd

This is the **BQ24072 (Power-Path Linear Charger)** alternative build path.

## Key Differences from Standard (BQ25896)
- **U3:** BQ24072 (VQFN-16, linear + power-path) instead of BQ25896 (WQFN-24, buck charger).
- **Power-Path:** YES. Can run radio while charging. This is the key advantage over MCP73831.
- **No I2C:** Charge current set via external resistor. No software control.
- **Moderate Complexity:** More pins than MCP73831, fewer than BQ25896.
- **Impact:** Best choice if you need power-path but want to avoid I2C complexity.

## Net Changes
- **I2C_SDA / I2C_SCL:** U3 no longer connects to I2C. U4 (fuel gauge) remains on I2C.
- **CHG_INT:** Removed.
- **STAT:** U3 has #PG (Power Good) and #CHG (Charge Status) pins.
- **PROG/ILIM:** Set via external resistors.

## All Other Nets
See `../standard/SCHEMATIC_PLAN.md` for the full net list.
