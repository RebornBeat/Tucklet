#!/usr/bin/env python3
"""
Tucklet hardware generator — the parametric "recipe" for every board variant.

This is the single source of truth for the electrical design. Running it emits,
for each physical board variant (radio x storage x form_factor), a complete set of build
artifacts under variants/<name>/:

    SPEC.md              human spec for this exact board
    BOM.csv              bill of materials for this exact board
    PIN_MAP.md           signal -> MCU pin mapping (datasheet step flagged)
    SCHEMATIC_PLAN.md    net-by-net connection plan
    tucklet-<name>.net   KiCad-importable netlist (Pcbnew: File > Import Netlist)
    block_diagram.svg    per-variant system block diagram

Design philosophy (mirrors the CAD round-trip workflow): the geometry/artifact
is generated from a parametric model, never hand-maintained. Edit THIS file and
re-run to regenerate every variant consistently.

  python3 gen_hardware.py            # generate all variants + common BOM
  python3 gen_hardware.py --check    # generate to a temp dir and validate only

HONESTY NOTE ON PIN NUMBERS: discrete ICs with fixed, well-known pinouts
(MCP73831, MAX17048, USB-C receptacle, CC resistors) use real pin numbers.
The MCU module's GPIO assignments are expressed as SIGNAL NAMES,
because the exact module pin numbers must be read from the current datasheet.
The netlist is a "logical" netlist in the KiCad sense.

License: CC BY-NC-SA 4.0 (hardware design files).
"""

from __future__ import annotations
import csv
import io
import os
import sys
import argparse
from dataclasses import dataclass, field
from typing import Optional

# ---------------------------------------------------------------------------
# Variant model
# ---------------------------------------------------------------------------

# Radio definitions now include Chip Type derivation
# "label", "radios", "wireless_mb_s", "chip"
RADIOS = {
    "singlec5": {"label": "Single ESP32-C5", "radios": 1, "wireless_mb_s": 9, "chip": "c5"},
    "dualc5":   {"label": "Dual ESP32-C5 (link-aggregated, experimental)", "radios": 2, "wireless_mb_s": 15, "chip": "c5"},
    "singlee22": {"label": "Single ESP32-E22 (Wi-Fi 6E)", "radios": 1, "wireless_mb_s": 150, "chip": "e22"},
    "duale22":   {"label": "Dual ESP32-E22 (High-Performance)", "radios": 2, "wireless_mb_s": 300, "chip": "e22"},
}

# eMMC capacities offered; cost is a small-volume commercial estimate (VERIFY on
# LCSC/Mouser/Arrow). microSD has no flash cost (customer supplies the card).
EMMC_CAPACITIES_GIB = [8, 16, 32, 64, 128, 256]
EMMC_COST = {8: 2.50, 16: 3.50, 32: 5.00, 64: 8.00, 128: 13.00, 256: 22.00}

STORAGE = {
    "microsd": {"label": "microSD (swappable)", "kind": "microSd"},
    "emmc":    {"label": "eMMC (sealed)", "kind": "Emmc"},
}

FORM_FACTORS = {
    "wroom": {"label": "Standard (WROOM)", "size_note": "Standard footprint"},
    "mini":  {"label": "Mini (Compact)", "size_note": "Ultra-compact footprint"},
}

# Enclosure envelopes (mm) from mechanical/ and docs/TRANSFER_PERFORMANCE.md.
ENVELOPE = {
    "microsd": (35, 28, 9),
    "emmc":    (32, 24, 8),
}


@dataclass
class Variant:
    radio: str       # "singlec5" | "dualc5" | "singlee22" | "duale22"
    storage: str     # "microsd" | "emmc"
    form_factor: str # "wroom" | "mini"

    @property
    def name(self) -> str:
        return f"{self.radio}-{self.form_factor}-{self.storage}"

    @property
    def radios(self) -> int:
        return RADIOS[self.radio]["radios"]

    @property
    def is_dual(self) -> bool:
        return self.radios == 2

    @property
    def is_emmc(self) -> bool:
        return self.storage == "emmc"

    @property
    def wireless_mb_s(self) -> int:
        return RADIOS[self.radio]["wireless_mb_s"]

    @property
    def chip(self) -> str:
        return RADIOS[self.radio]["chip"]

    @property
    def is_e22(self) -> bool:
        return self.chip == "e22"


# Generate ALL variants: 4 Radio configs x 2 Storage x 2 Form Factors = 16 Variants
ALL_VARIANTS = [Variant(r, s, f) for r in RADIOS for s in STORAGE for f in FORM_FACTORS]


# ---------------------------------------------------------------------------
# Component + net model (KiCad netlist primitives)
# ---------------------------------------------------------------------------

@dataclass
class Comp:
    ref: str
    value: str
    footprint: str
    desc: str
    # pins: list of (pad/number-or-signal, pinfunction, pintype)
    pins: list = field(default_factory=list)
    # BOM
    unit_cost: float = 0.0
    qty: int = 1
    mpn: str = ""
    note: str = ""


def build_components(v: Variant) -> list[Comp]:
    """Return the component list for a given variant."""
    c: list[Comp] = []

    # --- U1 radio/MCU (signal-named pins) ---
    # Logic to select specific MCU part
    if v.chip == "e22":
        base_val = "ESP32-E22"
        base_mpn = "ESP32-E22-WROOM-1" if v.form_factor == "wroom" else "ESP32-E22-MINI-1"
        # E22 Footprints (Hypothetical standard names - User must verify/create)
        base_fp = "RF_Module:Espressif_ESP32-E22_WROOM-1" if v.form_factor == "wroom" else "RF_Module:Espressif_ESP32-E22_MINI-1"
        mcu_desc = "Wi-Fi 6E + BLE 5.4 Radio Co-Processor (MCU mode)"
        mcu_cost = 4.50 if v.form_factor == "wroom" else 4.20
    else: # C5
        base_val = "ESP32-C5-WROOM-1" if v.form_factor == "wroom" else "ESP32-C5-MINI-1"
        base_mpn = f"{base_val}-N8"
        # Using S3 footprint as proxy for C5 (standard hacking practice until libs update)
        base_fp = "RF_Module:ESP32-S3-WROOM-1" if v.form_factor == "wroom" else "RF_Module:ESP32-S3-WROOM-1" # Mini footprint mapping TBD
        mcu_desc = "Wi-Fi 6 dual-band + BLE module (radio/MCU)"
        mcu_cost = 2.60 if v.form_factor == "wroom" else 2.40

    radio_pins = [
        ("GND", "GND", "power_in"),
        ("3V3", "VDD", "power_in"),
        ("EN", "EN", "input"),
        ("USB_DP", "USB_D+", "bidirectional"),
        ("USB_DM", "USB_D-", "bidirectional"),
        ("SD_CLK", "SDIO_CLK", "output"),
        ("SD_CMD", "SDIO_CMD", "bidirectional"),
        ("SD_D0", "SDIO_D0", "bidirectional"),
        ("SD_D1", "SDIO_D1", "bidirectional"),
        ("SD_D2", "SDIO_D2", "bidirectional"),
        ("SD_D3", "SDIO_D3", "bidirectional"),
        ("I2C_SDA", "I2C_SDA", "bidirectional"),
        ("I2C_SCL", "I2C_SCL", "output"),
        ("GAUGE_ALRT", "GPIO_ALRT", "input"),
        ("BTN", "GPIO_BTN", "input"),
        ("LED_DIN", "GPIO_LED", "output"),
        ("CHG_STAT", "GPIO_CHGSTAT", "input"),
        ("SD_SEL", "GPIO_SDSEL", "output"),
    ]
    if v.storage == "microsd":
        radio_pins.append(("SD_DET", "GPIO_SDDET", "input"))
    if v.is_dual:
        # Aggregation link
        radio_pins += [("AGG_TX", "AGG_A", "output"), ("AGG_RX", "AGG_B", "input")]

    c.append(Comp("U1", base_val, base_fp, mcu_desc,
                  pins=radio_pins, unit_cost=mcu_cost, mpn=base_mpn))

    if v.is_dual:
        radio2_pins = [
            ("GND", "GND", "power_in"),
            ("3V3", "VDD", "power_in"),
            ("EN", "EN", "input"),
            ("AGG_RX", "AGG_A", "input"),
            ("AGG_TX", "AGG_B", "output"),
        ]
        c.append(Comp("U1B", base_val, base_fp,
                      "Second radio for link aggregation",
                      pins=radio2_pins, unit_cost=mcu_cost, mpn=base_mpn))

    # --- U2 USB-HS storage bridge (real-ish pinout, generic) ---
    c.append(Comp("U2", "USB2.0-HS SD/eMMC bridge", "Package_DFN_QFN:QFN-24",
                  "USB High-Speed mass-storage bridge; owns storage when plugged in",
                  pins=[
                      ("1", "VBUS", "power_in"),
                      ("2", "USB_DP", "bidirectional"),
                      ("3", "USB_DM", "bidirectional"),
                      ("4", "3V3", "power_in"),
                      ("5", "GND", "power_in"),
                      ("6", "SD_CLK", "output"),
                      ("7", "SD_CMD", "bidirectional"),
                      ("8", "SD_D0", "bidirectional"),
                      ("9", "SD_D1", "bidirectional"),
                      ("10", "SD_D2", "bidirectional"),
                      ("11", "SD_D3", "bidirectional"),
                      ("12", "SD_SEL", "input"),
                  ], unit_cost=1.60, mpn="GL3224-OEM (or RTS5306-class)"))

    # --- U3 charger MCP73831 (real pinout, SOT-23-5) ---
    c.append(Comp("U3", "MCP73831", "Package_TO_SOT_SMD:SOT-23-5",
                  "Single-cell Li-ion/LiPo charge management",
                  pins=[
                      ("1", "STAT", "open_collector"),
                      ("2", "VSS", "power_in"),
                      ("3", "VBAT", "power_out"),
                      ("4", "VDD", "power_in"),
                      ("5", "PROG", "passive"),
                  ], unit_cost=0.50, mpn="MCP73831T-2ACI/OT"))

    # --- U4 fuel gauge MAX17048 (real pinout, TDFN-8) ---
    c.append(Comp("U4", "MAX17048", "Package_DFN_QFN:TDFN-8-1EP_3x3mm",
                  "I2C fuel gauge (real battery percent)",
                  pins=[
                      ("1", "VDD", "power_in"),
                      ("2", "CTG", "passive"),
                      ("3", "QSTRT", "input"),
                      ("4", "GND", "power_in"),
                      ("5", "ALRT", "open_collector"),
                      ("6", "SCL", "input"),
                      ("7", "SDA", "bidirectional"),
                      ("8", "CELL", "passive"),
                  ], unit_cost=0.80, mpn="MAX17048G+T10"))

    # --- U5 buck 3V3 (sized for radio peak) ---
    # E22 requires significantly more power (Dual core 500MHz + 6E Radio)
    if v.is_e22:
        buck_cost = 1.20
        buck_val = "3V3 buck (>=2A for E22)"
        buck_mpn = "TPS62840-class"
    elif v.is_dual:
        buck_cost = 0.55
        buck_val = "3V3 buck (>=1A for dual)"
        buck_mpn = "TPS62740-class"
    else:
        buck_cost = 0.40
        buck_val = "3V3 buck (>=600mA)"
        buck_mpn = "TPS62740-class"

    c.append(Comp("U5", buck_val, "Package_TO_SOT_SMD:SOT-23-6",
                  "Step-down regulator, sized for Wi-Fi TX peak",
                  pins=[
                      ("1", "VIN", "power_in"),
                      ("2", "GND", "power_in"),
                      ("3", "EN", "input"),
                      ("4", "FB", "passive"),
                      ("5", "SW", "passive"),
                      ("6", "VOUT", "power_out"),
                  ], unit_cost=buck_cost, mpn=buck_mpn))

    # --- U6 SD bus mux / ownership handoff ---
    c.append(Comp("U6", "SD 2:1 bus mux", "Package_DFN_QFN:QFN-20",
                  "Arbitrates microSD/eMMC bus between radio (A) and bridge (B)",
                  pins=[
                      ("SEL", "SEL", "input"),
                      ("GND", "GND", "power_in"),
                      ("VCC", "3V3", "power_in"),
                      ("C_CLK", "SD_CLK", "bidirectional"),
                      ("C_CMD", "SD_CMD", "bidirectional"),
                      ("C_D0", "SD_D0", "bidirectional"),
                      ("C_D1", "SD_D1", "bidirectional"),
                      ("C_D2", "SD_D2", "bidirectional"),
                      ("C_D3", "SD_D3", "bidirectional"),
                  ], unit_cost=0.45, mpn="TS3A-class 2:1"))

    # --- J1 USB-C receptacle (USB2.0 subset) ---
    c.append(Comp("J1", "USB-C receptacle", "Connector_USB:USB_C_Receptacle_USB2.0_16P",
                  "Charge + wired data + CC",
                  pins=[
                      ("A4", "VBUS", "power_out"), ("B4", "VBUS", "power_out"),
                      ("A9", "VBUS", "power_out"), ("B9", "VBUS", "power_out"),
                      ("A1", "GND", "power_in"), ("B1", "GND", "power_in"),
                      ("A12", "GND", "power_in"), ("B12", "GND", "power_in"),
                      ("A6", "USB_DP", "bidirectional"), ("B6", "USB_DP", "bidirectional"),
                      ("A7", "USB_DM", "bidirectional"), ("B7", "USB_DM", "bidirectional"),
                      ("A5", "CC1", "passive"), ("B5", "CC2", "passive"),
                  ], unit_cost=0.30, mpn="USB4105-GF-A-060 (or equiv)"))

    # --- D1 USB ESD array ---
    c.append(Comp("D1", "USB ESD array", "Package_TO_SOT_SMD:SOT-23-6",
                  "TVS protection on D+/D-/VBUS",
                  pins=[("1", "USB_DP", "passive"), ("2", "USB_DM", "passive"),
                        ("3", "GND", "power_in"), ("6", "VBUS", "passive")],
                  unit_cost=0.20, mpn="USBLC6-2SC6"))

    # --- CC resistors ---
    c.append(Comp("R1", "5.1k", "Resistor_SMD:R_0402_1005Metric",
                  "USB-C CC1 pulldown (sink role)",
                  pins=[("1", "CC1", "passive"), ("2", "GND", "passive")],
                  unit_cost=0.01, mpn="RC0402FR-075K1L"))
    c.append(Comp("R2", "5.1k", "Resistor_SMD:R_0402_1005Metric",
                  "USB-C CC2 pulldown (sink role)",
                  pins=[("1", "CC2", "passive"), ("2", "GND", "passive")],
                  unit_cost=0.01, mpn="RC0402FR-075K1L"))

    # --- PROG resistor for charger current ---
    c.append(Comp("R3", "2k", "Resistor_SMD:R_0402_1005Metric",
                  "MCP73831 PROG (sets ~500mA charge; size to cell)",
                  pins=[("1", "PROG", "passive"), ("2", "GND", "passive")],
                  unit_cost=0.01, mpn="RC0402FR-072KL"))

    # --- I2C pull-ups ---
    c.append(Comp("R4", "4.7k", "Resistor_SMD:R_0402_1005Metric",
                  "I2C SDA pull-up",
                  pins=[("1", "I2C_SDA", "passive"), ("2", "3V3", "passive")],
                  unit_cost=0.01, mpn="RC0402FR-074K7L"))
    c.append(Comp("R5", "4.7k", "Resistor_SMD:R_0402_1005Metric",
                  "I2C SCL pull-up",
                  pins=[("1", "I2C_SCL", "passive"), ("2", "3V3", "passive")],
                  unit_cost=0.01, mpn="RC0402FR-074K7L"))

    # --- Button pull-up + button ---
    c.append(Comp("R6", "10k", "Resistor_SMD:R_0402_1005Metric",
                  "Button pull-up",
                  pins=[("1", "BTN", "passive"), ("2", "3V3", "passive")],
                  unit_cost=0.01, mpn="RC0402FR-0710KL"))
    c.append(Comp("SW1", "tactile button", "Button_Switch_SMD:SW_SPST_B3U-1000P",
                  "Pair / factory-reset (press patterns)",
                  pins=[("1", "BTN", "passive"), ("2", "GND", "passive")],
                  unit_cost=0.10, mpn="B3U-1000P"))

    # --- Status LED ---
    c.append(Comp("LED1", "WS2812B", "LED_SMD:LED_WS2812B_PLCC4_5.0x5.0mm",
                  "Addressable RGB status LED",
                  pins=[("VDD", "3V3", "power_in"), ("GND", "GND", "power_in"),
                        ("DIN", "LED_DIN", "input"), ("DOUT", "NC_LEDDOUT", "output")],
                  unit_cost=0.20, mpn="WS2812B-2020"))

    # --- SD pull-ups (CMD + data) ---
    for i, sig in enumerate(["SD_CMD", "SD_D0", "SD_D1", "SD_D2", "SD_D3"]):
        c.append(Comp(f"R{7+i}", "10k", "Resistor_SMD:R_0402_1005Metric",
                      f"{sig} pull-up",
                      pins=[("1", sig, "passive"), ("2", "3V3", "passive")],
                      unit_cost=0.01, mpn="RC0402FR-0710KL"))

    # --- Storage device ---
    if v.storage == "microsd":
        sd_pins = [
            ("CLK", "SD_CLK", "input"), ("CMD", "SD_CMD", "bidirectional"),
            ("DAT0", "SD_D0", "bidirectional"), ("DAT1", "SD_D1", "bidirectional"),
            ("DAT2", "SD_D2", "bidirectional"), ("DAT3", "SD_D3", "bidirectional"),
            ("VDD", "3V3", "power_in"), ("VSS", "GND", "power_in"),
            ("DET", "SD_DET", "passive"),
        ]
        c.append(Comp("J2", "microSD push-push socket",
                      "Connector_Card:microSD_HC_Hirose_DM3D-SF",
                      "User-swappable microSD (SDIO 4-bit)",
                      pins=sd_pins, unit_cost=0.40, mpn="DM3D-SF"))
    else:
        emmc_pins = [
            ("CLK", "SD_CLK", "input"), ("CMD", "SD_CMD", "bidirectional"),
            ("DAT0", "SD_D0", "bidirectional"), ("DAT1", "SD_D1", "bidirectional"),
            ("DAT2", "SD_D2", "bidirectional"), ("DAT3", "SD_D3", "bidirectional"),
            ("VCC", "3V3", "power_in"), ("VCCQ", "3V3", "power_in"),
            ("VSS", "GND", "power_in"), ("VSSQ", "GND", "power_in"),
            ("RST", "EMMC_RST", "input"),
        ]
        c.append(Comp("XU7", "eMMC 5.1 (capacity per SKU)",
                      "Package_BGA:eMMC_153Ball_TFBGA_11.5x13mm",
                      "Sealed managed NAND (JEDEC eMMC 5.1, 153-ball TFBGA)",
                      pins=emmc_pins, unit_cost=EMMC_COST[64], mpn="KLMxGyFExA-class",
                      note="capacity & cost set per SKU; 64GB shown as default"))

    # --- Battery connector / cell ---
    # E22 requires larger battery
    batt_desc = "LiPo 300-500mAh + PCM" if v.is_e22 else "LiPo 120-200mAh + PCM"
    batt_cost = 3.50 if v.is_e22 else 2.00
    c.append(Comp("BT1", batt_desc, "Connector:Conn_01x02_Pin",
                  "Single-cell LiPo with protection circuit",
                  pins=[("1", "VBAT", "power_out"), ("2", "GND", "power_in")],
                  unit_cost=batt_cost, mpn="cell + JST-ACH or solder pads"))

    # --- Bulk/decoupling (grouped, representative) ---
    c.append(Comp("C_GRP", "decoupling+bulk (grouped)", "Capacitor_SMD:C_0402_1005Metric",
                  "100nF per IC rail + 10uF bulk near U1/U5 + USB/charger input caps",
                  pins=[], unit_cost=0.50, qty=1,
                  note="represented as a group; expand to individual C refs in KiCad"))

    return c


def build_nets(v: Variant, comps: list[Comp]) -> dict[str, list[tuple[str, str]]]:
    """Collect nets by walking each component's pins."""
    nets: dict[str, list[tuple[str, str]]] = {}
    for comp in comps:
        for (pinid, signal, _ptype) in comp.pins:
            if signal.startswith("NC_"):
                continue
            nets.setdefault(signal, []).append((comp.ref, pinid))
    return nets


# ---------------------------------------------------------------------------
# Emitters
# ---------------------------------------------------------------------------

def emit_kicad_netlist(v: Variant, comps: list[Comp], nets: dict) -> str:
    """Emit a KiCad-format (version 'E') netlist."""
    out = io.StringIO()
    out.write('(export (version "E")\n')
    out.write('  (design\n')
    out.write(f'    (source "gen_hardware.py")\n')
    out.write(f'    (date "generated")\n')
    out.write(f'    (tool "tucklet gen_hardware.py")\n')
    out.write(f'    (sheet (number "1") (name "/{v.name}/") (tstamps "/")))\n')
    # components
    out.write('  (components\n')
    for cph in comps:
        if cph.ref == "C_GRP":
            continue
        out.write(f'    (comp (ref "{cph.ref}")\n')
        out.write(f'      (value "{cph.value}")\n')
        out.write(f'      (footprint "{cph.footprint}")\n')
        out.write(f'      (datasheet "")\n')
        out.write(f'      (fields (field (name "MPN") "{cph.mpn}") (field (name "Description") "{esc(cph.desc)}"))\n')
        out.write(f'      (libsource (lib "tucklet") (part "{esc(cph.value)}") (description "{esc(cph.desc)}"))\n')
        out.write(f'      (sheetpath (names "/") (tstamps "/"))\n')
        out.write(f'      (tstamps "{cph.ref}"))\n')
    out.write('  )\n')
    # nets
    out.write('  (nets\n')
    for code, (signal, nodes) in enumerate(sorted(nets.items()), start=1):
        if len(nodes) < 2:
            continue
        out.write(f'    (net (code "{code}") (name "{signal}")\n')
        for (ref, pinid) in nodes:
            out.write(f'      (node (ref "{ref}") (pin "{pinid}"))\n')
        out.write('    )\n')
    out.write('  )\n')
    out.write(')\n')
    return out.getvalue()


def esc(s: str) -> str:
    return s.replace('"', "'")


def emit_bom(v: Variant, comps: list[Comp]) -> str:
    out = io.StringIO()
    w = csv.writer(out)
    w.writerow(["ref", "qty", "value", "mpn", "footprint", "unit_cost_usd", "line_cost_usd", "description", "note"])
    total = 0.0
    for c in comps:
        line = round(c.unit_cost * c.qty, 3)
        total += line
        w.writerow([c.ref, c.qty, c.value, c.mpn, c.footprint,
                    f"{c.unit_cost:.2f}", f"{line:.2f}", c.desc, c.note])
    # PCB + enclosure rows
    w.writerow(["PCB", 1, "2-4 layer board", "", "", "1.50", "1.50", "Small board, volume pricing", ""])
    w.writerow(["ENC", 1, "enclosure (3D-print early)", "", "", "2.00", "2.00",
                f"{ENVELOPE[v.storage][0]}x{ENVELOPE[v.storage][1]}x{ENVELOPE[v.storage][2]}mm shell", "molding tooling is separate capex"])
    total_full = total + 1.50 + 2.00
    w.writerow([])
    w.writerow(["TOTAL_ELECTRONICS", "", "", "", "", "", f"{total:.2f}", "", ""])
    w.writerow(["TOTAL_FULL_BOM", "", "", "", "", "", f"{total_full:.2f}",
                "microSD card extra/bundled" if v.storage == "microsd" else "eMMC cost is for 64GB; see SPEC for all capacities", ""])
    return out.getvalue()


def emit_pin_map(v: Variant) -> str:
    det = "\n| SD_DET | card-detect | free GPIO | microSD insertion detect |" if v.storage == "microsd" else ""
    dual = ""
    if v.is_dual:
        dual = """
## Second radio (U1B) + aggregation link (Dual Radio only)
| Signal | Role | Pin (U1 / U1B) | Notes |
|---|---|---|---|
| AGG_TX / AGG_RX | radio<->radio coordination | free UART pair | U1.AGG_TX -> U1B.AGG_RX and vice-versa |
| 3V3 / GND / EN | power U1B | — | U5 must be sized for two radios' peak |
"""
    emmc_extra = ""
    if v.is_emmc:
        emmc_extra = """
| EMMC_RST | eMMC hardware reset | free GPIO | drive per JEDEC reset timing |
| (DAT4..DAT7) | optional 8-bit eMMC | free GPIOs | only if bridge + layout support 8-bit; raises wired throughput |
"""

    chip_name = "ESP32-E22" if v.is_e22 else "ESP32-C5"

    return f"""# Pin Map — {v.name} ({chip_name})

> **Read this first.** GPIO numbers are NOT hard-coded here because the
> {chip_name} pin table must be taken from the **current datasheet**.
> Assign each signal below to a free GPIO, keeping the **strapping pins clean**.

## Fixed-function (must use the dedicated pins)
| Signal | Function | Pin | Notes |
|---|---|---|---|
| USB_DP / USB_DM | native USB (flashing/JTAG; NOT the data path) | dedicated USB pins | route to test pads; storage data goes through the USB-HS bridge instead |
| 3V3 / GND / EN | power + enable | dedicated | EN via RC reset network |

## Storage bus — SDIO 4-bit (shared via U6 mux)
| Signal | Function | Pin | Notes |
|---|---|---|---|
| SD_CLK | SDIO clock | free GPIO | to U6 A-side |
| SD_CMD | SDIO command | free GPIO | 10k pull-up |
| SD_D0..D3 | SDIO data 0-3 | free GPIOs (contiguous preferred) | 10k pull-ups each; D3=CS in 1-bit/SPI fallback |
| SD_SEL | storage ownership select | free GPIO | drives U6; high=bridge owns (USB plugged), low=radio owns |{det}{emmc_extra}

## I2C (fuel gauge U4)
| Signal | Function | Pin | Notes |
|---|---|---|---|
| I2C_SDA | I2C data | free GPIO | 4.7k pull-up |
| I2C_SCL | I2C clock | free GPIO | 4.7k pull-up |
| GAUGE_ALRT | low-battery interrupt | free GPIO | from MAX17048 ALRT (open-drain) |

## UI + charger status
| Signal | Function | Pin | Notes |
|---|---|---|---|
| BTN | tactile button | free GPIO | 10k pull-up + firmware debounce; press patterns = pair / factory-reset |
| LED_DIN | WS2812 data | free GPIO | single addressable RGB |
| CHG_STAT | charger status | free GPIO | from MCP73831 STAT (open-drain) |
{dual}
## Routing note
Wi-Fi {'6E' if v.is_e22 else '5 GHz'} TX bursts are the current peak. Wide copper on VBUS/VBAT/3V3, bulk +
decoupling close to U1 (and U1B on dual), and size U5 for the peak, not the
average — this prevents mid-transfer brownout resets.
"""


def emit_schematic_plan(v: Variant, nets: dict) -> str:
    lines = [f"# Schematic Plan (net-by-net) — {v.name}",
             "",
             "Build this in KiCad: place the symbols from BOM.csv, then wire the nets",
             "below. Or import `tucklet-" + v.name + ".net` into Pcbnew to start the",
             "board with a ratsnest directly (logical netlist; reconcile module pins",
             "via PIN_MAP.md).",
             "",
             f"Variant: **{RADIOS[v.radio]['label']}**, **{STORAGE[v.storage]['label']}**, **{FORM_FACTORS[v.form_factor]['label']}**.",
             "",
             "## Power tree",
             "`VBUS` (USB-C) -> U3 charge in; `VBAT` (BT1 <-> U3 BAT <-> U4 CELL <-> U5 VIN);",
             "`+3V3` (U5 VOUT -> U1" + ("/U1B" if v.is_dual else "") + ", U2, U4, U6, storage, LED, pull-ups).",
             "Common `GND` pour, stitched.",
             "",
             "## Storage ownership (the shared-bus trick)",
             "Storage SD_* lines connect to the **common** side of U6. U1 SDIO connects",
             "to U6 A-side; U2 (USB-HS bridge) connects to U6 B-side. `SD_SEL` (driven by",
             "VBUS-present detection in firmware/hardware) selects the owner: plugged in =",
             "bridge owns (fast wired ~20-40 MB/s); otherwise radio owns (wireless). Never both.",
             "",
             "## All nets (signal -> nodes)",
             "",
             "| Net | Nodes (ref.pin) |",
             "|---|---|"]
    for signal, nodes in sorted(nets.items()):
        if len(nodes) < 2:
            continue
        nodestr = ", ".join(f"{r}.{p}" for (r, p) in nodes)
        lines.append(f"| `{signal}` | {nodestr} |")
    lines += ["",
              "## Build order in KiCad",
              "power -> U1" + ("/U1B + AGG link" if v.is_dual else "") +
              " -> storage + U6 mux -> U2 bridge -> I2C/UI -> charger/gauge. Export the",
              "netlist when done; that's the file to iterate from.",
              ""]
    return "\n".join(lines)


def emit_spec(v: Variant) -> str:
    ex, ey, ez = ENVELOPE[v.storage]

    # Dynamic sections based on variant
    if v.is_emmc:
        cap_rows = "\n".join(f"| {g} GB | ~${EMMC_COST[g]:.2f} | sealed |" for g in EMMC_CAPACITIES_GIB)
        storage_block = f"""## Storage — eMMC (sealed, per-SKU capacity)
153-ball TFBGA eMMC 5.1, 11.5 x 13.0 x 1.2 mm. Capacity is fixed at manufacture.

| Capacity | eMMC est. cost | Notes |
|---|---|---|
{cap_rows}
"""
    else:
        storage_block = """## Storage — microSD (swappable)
Push-push microSD socket, SDIO 4-bit. Customer supplies/upgrades the card.
"""

    if v.is_e22:
        chip_block = f"""
## Radio — ESP32-E22 (Wi-Fi 6E)
Tri-band Wi-Fi 6E (2.4, 5, 6 GHz) + BLE 5.4.
**High Performance:** ~{v.wireless_mb_s} MB/s theoretical wireless throughput.
**Power:** Requires larger battery and robust 3.3V regulation (U5).
**Thermal:** High sustained throughput generates heat; monitor enclosure temps.
"""
    else:
        chip_block = f"""
## Radio — ESP32-C5
Dual-band Wi-Fi 6 (2.4 + 5 GHz) + BLE. ~{v.wireless_mb_s} MB/s wireless at close range.
"""

    dual_block = ""
    if v.is_dual:
        dual_block = """
**Note:** Dual-radio link aggregation is experimental. Validate RF isolation between antennas.
"""

    return f"""# Variant Spec — {v.name}

**{RADIOS[v.radio]['label']} + {STORAGE[v.storage]['label']} ({FORM_FACTORS[v.form_factor]['label']})**

## What this board is
A Tucklet build with the {RADIOS[v.radio]['label'].lower()} radio and
{STORAGE[v.storage]['label'].lower()} storage in a {FORM_FACTORS[v.form_factor]['label']} form factor.
{chip_block}{dual_block}
{storage_block}

## Performance
- Everyday wireless: **~{v.wireless_mb_s} MB/s**.
- Bulk wired (USB-HS bridge): **~20-40 MB/s**.

## Dimensions
Enclosure envelope: **{ex} x {ey} x {ez} mm** (AirTag-class).

## Files in this directory
- `BOM.csv` — Full bill of materials.
- `PIN_MAP.md` — Signal -> MCU pin mapping.
- `SCHEMATIC_PLAN.md` — Net-by-net connection plan.
- `tucklet-{v.name}.net` — KiCad logical netlist.
- `block_diagram.svg` — System diagram.

## Bring-up checklist
- [ ] Reconcile signal->GPIO from the datasheet (PIN_MAP).
- [ ] Confirm USB-HS bridge part number + handoff mode.
- [ ] Validate SDIO timing.
- [ ] Measure real TX current; size BT1 cell.
- [ ] FCC/CE certification required (radios present).
"""


def emit_block_svg(v: Variant) -> str:
    """A compact, variant-specific block diagram."""
    storage_label = "microSD (J2)" if v.storage == "microsd" else "eMMC (XU7)"
    # Construct Radio Label based on variant
    if v.is_e22:
        radio_label = "ESP32-E22 (U1)" if not v.is_dual else "ESP32-E22 x2"
        radio_sub = "Wi-Fi 6E Tri-Band" if not v.is_dual else "Aggregated 6E"
    else:
        radio_label = "ESP32-C5 (U1)" if not v.is_dual else "ESP32-C5 x2"
        radio_sub = "BLE + Wi-Fi 5GHz" if not v.is_dual else "BLE + 2x Wi-Fi"

    dual_note = "" if not v.is_dual else '<text x="430" y="200" text-anchor="middle" font-size="10" fill="#b5654f" font-style="italic">+ U1B, AGG link</text>'

    return f'''<svg viewBox="0 0 900 540" xmlns="http://www.w3.org/2000/svg" font-family="system-ui, sans-serif">
  <defs>
    <filter id="s" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="2" stdDeviation="4" flood-color="#7a5a48" flood-opacity="0.16"/></filter>
    <marker id="a" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="#7a5a48"/></marker>
  </defs>
  <rect width="900" height="540" fill="#fbf7f1"/>
  <text x="40" y="44" font-size="22" font-weight="bold" fill="#3a2c23">Tucklet — {v.name}</text>
  <text x="40" y="66" font-size="12.5" fill="#8a7464">{RADIOS[v.radio]['label']} · {STORAGE[v.storage]['label']} · {FORM_FACTORS[v.form_factor]['label']}</text>

  <g filter="url(#s)"><rect x="40" y="108" width="130" height="60" rx="12" fill="#fff"/></g>
  <text x="105" y="134" text-anchor="middle" font-size="13" font-weight="bold" fill="#3a2c23">USB-C (J1)</text>
  <text x="105" y="153" text-anchor="middle" font-size="10.5" fill="#6f5b4c">charge + data</text>
  <g filter="url(#s)"><rect x="40" y="204" width="130" height="56" rx="12" fill="#fff"/></g>
  <text x="105" y="228" text-anchor="middle" font-size="12" font-weight="bold" fill="#3a2c23">Charger U3</text>
  <text x="105" y="246" text-anchor="middle" font-size="10.5" fill="#6f5b4c">MCP73831</text>
  <g filter="url(#s)"><rect x="40" y="296" width="130" height="56" rx="12" fill="#fff"/></g>
  <text x="105" y="320" text-anchor="middle" font-size="12" font-weight="bold" fill="#3a2c23">LiPo BT1</text>
  <text x="105" y="338" text-anchor="middle" font-size="10.5" fill="#6f5b4c">+ gauge U4</text>
  <g filter="url(#s)"><rect x="40" y="388" width="130" height="52" rx="12" fill="#fff"/></g>
  <text x="105" y="411" text-anchor="middle" font-size="12" font-weight="bold" fill="#3a2c23">Buck U5 3V3</text>

  <g filter="url(#s)"><rect x="330" y="108" width="200" height="100" rx="14" fill="#fff"/></g>
  <text x="430" y="138" text-anchor="middle" font-size="14" font-weight="bold" fill="#3a2c23">{radio_label}</text>
  <text x="430" y="160" text-anchor="middle" font-size="11" fill="#6f5b4c">{radio_sub}</text>
  <text x="430" y="178" text-anchor="middle" font-size="11" fill="#6f5b4c">SDIO · I2C · GPIO</text>
  {dual_note}

  <g filter="url(#s)"><rect x="360" y="262" width="140" height="58" rx="12" fill="#fff"/></g>
  <text x="430" y="286" text-anchor="middle" font-size="12.5" font-weight="bold" fill="#3a2c23">SD mux U6</text>
  <text x="430" y="304" text-anchor="middle" font-size="10.5" fill="#6f5b4c">radio &lt;-&gt; bridge</text>

  <g filter="url(#s)"><rect x="330" y="372" width="200" height="72" rx="12" fill="#fff"/></g>
  <text x="430" y="400" text-anchor="middle" font-size="13" font-weight="bold" fill="#3a2c23">{storage_label}</text>
  <text x="430" y="420" text-anchor="middle" font-size="10.5" fill="#6f5b4c">SDIO 4-bit</text>

  <g filter="url(#s)"><rect x="690" y="262" width="170" height="58" rx="12" fill="#fff"/></g>
  <text x="775" y="286" text-anchor="middle" font-size="12.5" font-weight="bold" fill="#3a2c23">USB-HS bridge U2</text>
  <text x="775" y="304" text-anchor="middle" font-size="10.5" fill="#6f5b4c">~20-40 MB/s wired</text>

  <g filter="url(#s)"><rect x="690" y="108" width="170" height="100" rx="14" fill="#fff"/></g>
  <text x="775" y="138" text-anchor="middle" font-size="13" font-weight="bold" fill="#3a2c23">UI</text>
  <text x="775" y="162" text-anchor="middle" font-size="11" fill="#6f5b4c">Button SW1</text>
  <text x="775" y="180" text-anchor="middle" font-size="11" fill="#6f5b4c">RGB LED1</text>

  <g stroke="#7d9b86" stroke-width="2.5" fill="none" marker-end="url(#a)">
    <path d="M105,168 L105,204"/><path d="M105,260 L105,296"/><path d="M105,352 L105,388"/>
    <path d="M170,414 C 250,414 250,170 328,165"/>
    <path d="M170,416 C 300,470 560,470 688,300"/>
  </g>
  <g stroke="#7a5a48" stroke-width="2" fill="none" marker-end="url(#a)">
    <path d="M430,208 L430,260"/><path d="M430,320 L430,372"/>
    <path d="M500,292 L688,292"/>
    <path d="M170,138 C 250,120 600,120 688,150"/>
    <path d="M530,150 C 600,150 640,150 688,150"/>
  </g>
  <text x="40" y="500" font-size="11" fill="#8a7464" font-style="italic">Wired: USB-C -&gt; bridge -&gt; storage (bypasses radio). Wireless: phone &lt;-&gt; radio &lt;-&gt; storage. Never both at once.</text>
</svg>
'''


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

def generate(root: str) -> list[str]:
    written = []
    variants_dir = os.path.join(root, "variants")
    os.makedirs(variants_dir, exist_ok=True)
    for v in ALL_VARIANTS:
        vdir = os.path.join(variants_dir, v.name)
        os.makedirs(vdir, exist_ok=True)
        comps = build_components(v)
        nets = build_nets(v, comps)

        files = {
            "SPEC.md": emit_spec(v),
            "BOM.csv": emit_bom(v, comps),
            "PIN_MAP.md": emit_pin_map(v),
            "SCHEMATIC_PLAN.md": emit_schematic_plan(v, nets),
            f"tucklet-{v.name}.net": emit_kicad_netlist(v, comps, nets),
            "block_diagram.svg": emit_block_svg(v),
        }
        for fname, content in files.items():
            path = os.path.join(vdir, fname)
            with open(path, "w") as f:
                f.write(content)
            written.append(path)
    return written


def validate(root: str) -> None:
    """Lightweight self-checks on the generated artifacts."""
    import re
    problems = []
    for v in ALL_VARIANTS:
        vdir = os.path.join(root, "variants", v.name)
        net = os.path.join(vdir, f"tucklet-{v.name}.net")
        if not os.path.exists(net):
            problems.append(f"{v.name}: netlist missing")
            continue

        txt = open(net).read()
        # balanced parens
        if txt.count("(") != txt.count(")"):
            problems.append(f"{v.name}: unbalanced parens in netlist")
        # every net has >=2 nodes
        for m in re.finditer(r'\(net \(code "\d+"\) \(name "([^"]+)"\)(.*?)\n    \)', txt, re.S):
            name, body = m.group(1), m.group(2)
            if body.count("(node") < 2:
                problems.append(f"{v.name}: net {name} has <2 nodes")
        # GND and 3V3 present
        for required in ["GND", "3V3", "VBUS", "VBAT", "SD_CLK", "USB_DP"]:
            if f'(name "{required}")' not in txt:
                problems.append(f"{v.name}: missing net {required}")
        # dual has aggregation crossover nets
        if v.is_dual and ('(name "AGG_A")' not in txt or '(name "AGG_B")' not in txt):
            problems.append(f"{v.name}: dual variant missing AGG_A/AGG_B crossover")
        # storage device present
        dev = "J2" if v.storage == "microsd" else "XU7"
        if f'(ref "{dev}")' not in txt:
            problems.append(f"{v.name}: missing storage device {dev}")

    if problems:
        print("VALIDATION FAILED:")
        for p in problems:
            print("  -", p)
        sys.exit(1)
    print(f"VALIDATION OK: {len(ALL_VARIANTS)} variants, all netlists balanced, "
          f"every net >=2 nodes, power/USB/SD/storage present.")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="generate to ./_check and validate only")
    ap.add_argument("--root", default=os.path.dirname(os.path.abspath(__file__)))
    args = ap.parse_args()
    root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_check") if args.check else args.root
    written = generate(root)
    validate(root)
    print(f"Generated {len(written)} files under {os.path.join(root, 'variants')}")
    for w in written:
        print("  ", os.path.relpath(w, root))


if __name__ == "__main__":
    main()
