# 📖 THE TUCKLET HARDWARE MASTER REFERENCE DOCUMENT
**Project:** Tucklet Charm (Standard & Pro Variants)
**Baseline Variant:** `singlec5-wroom-microsd`
**Status:** Finalized & Verified
**License:** CC BY-NC-SA 4.0

This document is the single source of truth for the Tucklet Hardware component selection. It captures all engineering decisions, supply chain corrections, technical insights, and legal frameworks established during the component verification phase.

---

## 🚨 SECTION 1: CRITICAL ENGINEERING WARNINGS & TRAPS AVOIDED

During the component hunt, several significant traps were identified in generic/placeholder BOMs. **Do not revert these decisions.**

1.  **The "AP6212" Trap (U5 Regulator):**
    *   **The Myth:** Many online guides reference the "AP6212" as a cheap 1.5A buck regulator for ESP32.
    *   **The Reality:** On JLCPCB/LCSC, the part number `AP6212` is *exclusively* a WiFi module (QFN-44). Placing this order will result in a WiFi chip being placed where your power regulator should go, resulting in a dead short.
    *   **The Fix:** We use the **SY8089AAC** (1.5A, SOT-23-5) instead.
2.  **The SD Mux "Pin Math" Trap (U6):**
    *   **The Myth:** A 20-pin QFN package can support a 6-channel SDIO 2:1 Mux.
    *   **The Reality:** Switching 4-bit SDIO (CLK, CMD, D0-D3) requires 6 signals × 3 paths (Common, A, B) + Power/GND/SEL = ~24 pins minimum. A 20-pin package is physically impossible for this function.
    *   **The Fix:** We use the **TXS02612** (QFN-24). Furthermore, the alternative `TMUX1574` was rejected because it is only a 4-channel mux (insufficient for SDIO 4-bit).
3.  **The TPS62740 Under-rating (U5 Regulator):**
    *   **The Myth:** TPS62740 is a good low-power buck for ESP32.
    *   **The Reality:** It outputs 300-400mA. ESP32 Wi-Fi TX spikes exceed 500mA. This chip will cause brownout resets.
    *   **The Fix:** We use the **SY8089AAC** (1.5A) for standard, and **TPS62840DLCR** (750mA, ultra-low Iq) only for single-C5 battery-life variants.
4.  **The HXY MOSFET Clone Standby Drain (U3 Charger Alt):**
    *   **The Reality:** The HXY clone of the MCP73831 has a standby current of **200 µA**, compared to the Microchip original's **53 µA**. For a battery-powered charm sitting in deep sleep, the clone will drain the battery 4x faster.
    *   **The Fix:** Only specify the Microchip original (`C424093`) as the primary budget alternative.
5.  **Antenna Physics in Small Enclosures (U1):**
    *   **The Reality:** The standard ESP32-C5-WROOM PCB antenna is ~40-50% efficient in free space, but drops to ~10-20% when placed directly next to a LiPo battery (which acts as an RF shield).
    *   **The Fix:** For standard prototypes, the internal antenna is acceptable (phone-range). For "Pro" variants, the `1U` (U.FL connector) variant must be used with an external FPC antenna routed away from the battery. The ~0.5dB cable loss is vastly offset by clearing the battery obstruction.
6. *   **With PSRAM (`R8`):** You have 8MB of extra RAM. You can load a huge chunk of a photo/video into memory and blast it over WiFi. **Fast Transfers.**
*   **Without PSRAM (`N4` only):** You are limited to the ~400KB internal RAM of the C5 chip. You have to move files in tiny chunks. **Slower Transfers.**
---

## 🏆 SECTION 2: PRIMARY BASELINE BOM (`singlec5-wroom-microsd`)

This is the exact BOM for the standard variant. All parts are verified for JLCPCB assembly.

| Ref | Part Number | Description | Package | LCSC Code | Assembly | Est. Price (1u) | Critical Insights & Actions |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **U1** | **ESP32-C5-WROOM-1-N8R8** | **MCU + WiFi 6** | SMD 27.5x18mm | **C51950749** | Standard | $5.75 | **Primary Choice.** 8MB Flash + **8MB PSRAM** is critical for large file transfer buffers. |
| **U2** | **GL3224-OIY04** | **USB 3.0 Bridge** | QFN-32-EP(5x5) | **C157357** | Econ/Std | $1.02 | **Custom Part.** High-speed wired path (~70MB/s). |
| **U3** | **BQ25896RTWT** | **Power-Path Charger** | WQFN-24-EP(4x4) | **C2876371** | Econ/Std | $3.64 | **I2C Control.** Power-path allows running radio while charging. |
| **U4** | **MAX17048G+T10** | **Fuel Gauge** | DFN-8-EP(2x2) | **C2682616** | Econ/Std | $2.16 | **ModelGauge.** No sense resistor required. 1S only. |
| **U5** | **SY8089AAC** | **1.5A Buck Regulator** | SOT-23-5 | **C5187495** | Econ/Std | $0.17 | **CRITICAL FIX.** Replaces TPS62740/AP6212. 1.5A covers Single & Dual C5 peaks. |
| **U6** | **TXS02612RTWR** | **SD/SDIO 6-Bit Mux** | WQFN-24-EP(4x4) | **C140276** | Econ/Std | $0.70 | **CRITICAL FIX.** Replaces invalid "QFN-20" placeholder. 6-Channels required for SDIO 4-bit. Includes integrated pull-ups. |
| **J1** | **USB4105-GF-A-060** | **USB-C Receptacle** | SMD 16-Pin | **C3025063** | Econ/Std | $1.07 | USB 2.0 + CC Resistors. High assembly difficulty. |
| **J2** | **DM3D-SF** | **microSD Socket** | SMD Push-Push | **C719027** | Econ/Std | $1.41 | SDIO 4-bit capable. |
| **LED1** | **WS2812B-2020-V6** | **RGB Status LED** | SMD2020-4P (2x2mm)| **C52917434** | Econ/Std | $0.10 | **CRITICAL FIX.** Replaces 5x5mm version. "Economic Assembly" saves cost. V6 revision. |
| **SW1** | **B3U-1000P** | **Tactile Button** | SMD 3x2.5mm | **C231329** | Econ/Std | $0.18 | Standard SPST. |
| **BT1** | **BM02B-ACHSS-GAN-ETF** | **LiPo Connector** | SMD 2-Pin RA (1.2mm) | **C5118738** | **JST Name-Brand.** Superior to generic. Economic Assembly. |



---

## 🔄 SECTION 3: THE COMPLETE ALTERNATIVES MATRIX

This matrix ensures supply chain resilience and defines the exact components required for variant branching (Mini, Pro, 2S Battery). Substitutions must follow these constraints strictly to maintain schematic/footprint compatibility.

### 1. U1 Alternatives (MCU / Radio)
*Primary: `ESP32-C5-WROOM-1-N8R8`*

| Part Number | Role | LCSC Code | Assembly | Price (1u) | Insights & Warnings |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ESP32-C5-WROOM-1-N16R4** | **Avail Alt** | C49296926 | Econ/Std | $7.40 | 16MB Flash, 4MB PSRAM. Economic Assembly available. Good if N8R8 out of stock. |
| **ESP32-C5-WROOM-1-N16R8** | **Pro Perf** | C53054799 | Econ/Std | $6.02 | 16MB Flash + **8MB PSRAM**. Max performance for large buffers. |
| **ESP32-C5-WROOM-1-N4** | **Cost Alt** | C51950745 | Econ/Std | $9.23 | 4MB Flash, No PSRAM. **Warning:** Tiny buffers will result in significantly slower file transfers. |
| **ESP32-C5-WROOM-1** | **Base Alt** | C42441836 | Standard | $14.86 | Default Flash/PSRAM. Expensive. Standard Only assembly. |
| **ESP32-C5-WROOM-1U-N8R8** | **Pro Primary** | C51950748 | Econ/Std | $5.93 | **U.FL Connector.** Use for Pro variants to bypass battery RF shielding. |
| **ESP32-C5-WROOM-1U-N16R8** | **Pro Max** | C53069375 | Standard | $8.97 | **U.FL Connector.** Max storage + RAM + External Antenna. |
| **ESP32-C5-WROOM-1U-N8R4** | **Pro Alt** | C49308183 | Standard | $6.39 | **U.FL Connector.** 8MB Flash + 4MB PSRAM. |
| **ESP32-C5-WROOM-1U** | **Pro Base** | C48533540 | Standard | $8.97 | **U.FL Connector.** Default Flash/PSRAM. |
| **ESP32-C5-MINI-1-N4** | **Mini Var** | C50346179 | Standard | $8.97 | Required for "Nano" enclosure. **Warning:** Only 4MB Flash, No PSRAM. Slower transfers. |

### 2. U2 Alternatives (USB Bridges)
*Primary: `GL3224-OIY04` (USB 3.0)*

| Part Number | Role | LCSC Code | Package | Insights & Warnings |
| :--- | :--- | :--- | :--- | :--- |
| **GL823-QFN24** | **Size Alt** | C2848036 | QFN-24(4x4) | **Primary Alt.** USB 2.0 (25-35 MB/s). Fits tight spaces. Same 4x4mm size as other QFNs. |
| **GL823** | **Large Alt** | C48653 | SSOP-24 | **Valid Backup.** USB 2.0. Larger footprint (gull-wing leads). Use if QFN out of stock. |
| **PL2732** | **Supply Alt** | *Search* | QFN-32(5x5) | USB 3.0 (70-100+ MB/s). Excellent eMMC support. Pin-compat verification required. |
| **AU6601** | **Last Resort** | *Search* | QFN-48(7x7) | USB 3.0. Much larger footprint. Use only if GL3224/PL2732 unavailable. |

### 3. U3 Alternatives (Chargers)
*Primary: `BQ25896RTWT` (I2C, Power-Path)*

| Part Number | Role | LCSC Code | Package | Insights & Warnings |
| :--- | :--- | :--- | :--- | :--- |
| **MCP73831T-2ACI/OT** | **Budget Alt** | C424093 | SOT-23-5 | **Simplest.** No Power-Path, No I2C. Charge set by resistor. **Warning:** Must use Microchip original (53µA standby). HXY clones drain 4x faster (200µA). |
| **BQ24052DSQR** | **Feature Alt** | C3682337 | WSON-10(2x2) | **Power-Path.** Run radio while charging. No I2C. **Warning: 800mA limit. Single C5 variants ONLY.** |
| **BQ25887RGER** | **Pro Line** | C2761614 | QFN-24(4x4) | **CRITICAL for 2S Battery.** Boost charger + cell balancing. I2C. |

### 4. U4 Alternatives (Fuel Gauge)
*Primary: `MAX17048G+T10` (1S, ModelGauge)*

| Part Number | Role | LCSC Code | Package | Insights & Warnings |
| :--- | :--- | :--- | :--- | :--- |
| **BQ27421YZFR-G1A** | **Pro/2S** | C139621 | DSBGA-9 | **Primary Alt.** Impedance Track (more accurate). Cheapest option. Standard default profile. **Requires sense resistor.** |
| **BQ27421YZFR-G1B** | **Backup Alt** | C544661 | DSBGA-9 | Valid hardware. Use if G1A out of stock. Different factory battery profile. |

### 5. U5 Alternatives (Buck Regulator)
*Primary: `SY8089AAC` (1.5A)*

| Part Number | Role | LCSC Code | Package | Insights & Warnings |
| :--- | :--- | :--- | :--- | :--- |
| **TPS62840DLCR** | **Battery Alt** | C2071859 | VSON-8(1.5x2) | **Ultra-low Iq (60nA).** Maximizes deep sleep. **Warning: 750mA limit. Single C5 variants ONLY.** Do not use YBGR (BGA) version—it requires X-Ray. |

### 6. U6 Alternatives (SD Mux)
*Primary: `TXS02612RTWR` (Integrated Pull-ups)*

| Part Number | Role | LCSC Code | Package | Insights & Warnings |
| :--- | :--- | :--- | :--- | :--- |
| **TS3A27518ERTWR** | **Generic Alt** | C2651937 | WQFN-24(4x4) | **Primary Alt.** Fits same footprint as TXS02612. **Warning:** Requires external pull-up resistors (R7-R11 must be added to BOM). |
| **TS3A27518ETRTWRQ1** | **Price Alt** | C2673306 | QFN-24(4x4) | **Cheapest Option.** Fits same footprint. Automotive grade. Requires external pull-ups. |

### 7. XU7 (eMMC Storage - Sealed Variants)
*Required for "Sealed" variants. All require X-Ray inspection.*

| Part Number | Role | LCSC Code | Package | Insights & Warnings |
| :--- | :--- | :--- | :--- | :--- |
| **KLM8G1WEMB-B031** | **Base Storage** | C5359815 | FBGA-153 | New Arrival. 8GB. Standard eMMC 5.1 footprint. |
| **KLM8G1GETF-B041** | **Alt 8GB** | C499918 | FBGA-153 | Alternative 8GB source. $33.51 |
| **KLMAG1JETD-B041** | **16GB Var** | C499919 | FBGA-153 | 16GB Capacity. $34.44 |
| **KLMBG2JETD-B041** | **32GB Var** | C2803245 | FBGA-153 | 32GB Capacity. $50.02 |
| **FEMDNN064G-A3A56** | **64GB Var** | C5117595 | FBGA-153 | 64GB. Different manufacturer (FORESEE). $13.26 (Excellent Value). |

### 8. ANT1 (External Antenna - Pro/1U Variants)
*Required if using `1U` MCU variants.*

| Part Number | Role | LCSC Code | Insights & Warnings |
| :--- | :--- | :--- | :--- |
| **1461530300** | **Pro Primary** | C586441 | **Molex Dual-Band (2.4/5GHz).** REQUIRED for ESP32-C5. 5GHz WiFi will not work without this. $4.41 |
| **1461530200** | **Pro Alt** | C5872162 | **Molex Dual-Band.** Slightly different specs. $3.21 |
| **F7-2.4G** | **Budget Alt** | C403728 | **2.4GHz Only.** **Warning:** Cripples 5GHz WiFi. Use only for extreme cost saving. $0.41 |

---

## ⚡ SECTION 4: PASSIVES & DECOUPLING STRATEGY

Passives are commodity parts, but the *values* are dictated by the specific ICs chosen. Use standard JLCPCB "Basic" 0402 parts where possible to ensure Economic Assembly.

### Resistors
| Ref | Value | Package | LCSC Code | Function / Insight |
| :--- | :--- | :--- | :--- | :--- |
| **R1, R2** | **5.1kΩ** | 0402 | C25905 | USB-C CC Pull-downs (Required for USB-C spec). |
| **R3** | **2kΩ** | 0402 | C4109 | Charge Current Set (for MCP73831 alt). Check BQ25896 datasheet if using primary (I2C sets current). |
| **R4, R5** | **4.7kΩ** | 0402 | C25900 | I2C Bus Pull-ups (SDA/SCL). |
| **R6** | **10kΩ** | 0402 | C25744 | Button Pull-up (SW1). |
| **R7-R11** | **10kΩ** | 0402 | C25744 | **SD Card Pull-ups.** **CRITICAL:** If using Primary Mux (TXS02612), **OMIT THESE FROM BOM** (chip has internal pull-ups). If using Alt Mux (TS3A27518E), **INCLUDE THESE**. |

### Capacitors
| Ref | Value | Package | LCSC Code | Function / Insight |
| :--- | :--- | :--- | :--- | :--- |
| **C1-C4** | **100nF (0.1µF)** | 0402 | C1525 | Decoupling. Place near ESP32, Bridge, and Mux VCC pins. |
| **C5** | **10µF** | 0603 | C19702 | Bulk Capacitance. Place near battery input (SYS pin of charger). |
| **C6 (In)** | **10µF** | 0603 | C19702 | Regulator Input Cap (SY8089AAC requirement). |
| **C7 (Out)** | **22µF** | 0805 | C45783 | Regulator Output Cap. **Do not size down.** Required for SY8089AAC loop stability. |

---

## ⚖️ SECTION 5: LICENSING & LEGAL FRAMEWORK

**License Applied:** Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)

**What this means for the Hardware BOM:**
1.  **Design Protection:** The license protects the *Schematic Topology* (how the wires connect) and the *Implementation Files* (the `.kicad_sch` and netlists generated by `gen_hardware.py`).
2.  **Component Substitution:** A user cannot bypass the license by simply swapping a Samsung eMMC (Primary) for a FORESEE eMMC (Alt). Because the schematic topology remains identical, the resulting hardware is legally a **Derivative Work**.
3.  **ShareAlike Enforcement:** Any user who creates a derivative work (even by changing component manufacturers) must apply the same CC BY-NC-SA 4.0 license to their modified design. They cannot suddenly sell it commercially.
4.  **BOM Scope:** Listing 75 different eMMC chips in the official BOM is bad practice. This document lists the Primary and 2-3 Verified Alternatives. A legal clause should be added to the repository README: *"Any pin-compatible component may be substituted, but the resulting hardware remains a derivative work under the CC BY-NC-SA 4.0 license."*

---

*End of Master Reference Document. Proceed to Pin Extraction Workflow.*
