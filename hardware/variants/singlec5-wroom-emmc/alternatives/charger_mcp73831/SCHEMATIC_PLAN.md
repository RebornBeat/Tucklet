# Schematic Plan (MCP73831 Alternative) — singlec5-wroom-emmc

This is the **MCP73831 (Linear Charger)** alternative build path.

## Key Differences from Standard (BQ25896)
- **U3:** MCP73831 (SOT-23-5, simple linear charger) instead of BQ25896 (WQFN-24, buck charger).
- **No Power-Path:** Cannot run the radio while charging. System runs from battery only.
- **No I2C:** Charge current set via PROG resistor only. No software control.
- **Simpler:** Fewer pins, smaller footprint, lower cost.
- **Impact:** If user plugs in USB while transferring, the radio must pause or run from battery.
  This is acceptable for single-C5 variants but not recommended for dual/E22.

## Net Changes
- **I2C_SDA / I2C_SCL:** U3 no longer connects to I2C. U4 (fuel gauge) remains on I2C.
- **CHG_INT:** Removed (MCP73831 has no INT pin).
- **STAT:** U3 STAT pin connects to U1 CHG_STAT GPIO (same function, different pin).
- **PROG:** R3 sets charge current (same as standard).
- **TS:** Removed (MCP73831 has no thermistor input).
- **VBUS:** U3 VBUS connects directly (same as standard).

## All Other Nets
See `../standard/SCHEMATIC_PLAN.md` for the full net list.
