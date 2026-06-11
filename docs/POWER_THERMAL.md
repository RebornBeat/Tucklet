# Power, Thermal & Production-Readiness Engineering

Everything that has to be true for the device to be safe, reliable, and
manufacturable — not just functional on a bench.

## Power budget
The charm must sip power at rest (so it lasts weeks and feels "always there")
and tolerate WiFi current spikes during transfer without browning out.

| State | Dominant draw | Approx current | Notes |
|---|---|---|---|
| Asleep | RTC + leakage | ~10-50 uA | radios off, deep sleep, button/timer wake |
| Advertising (BLE low-duty) | BLE TX bursts | ~tens-hundreds uA avg | rotating address; weeks on a small cell |
| Connected (BLE only) | BLE link | ~3-8 mA | STATUS notifications |
| Transferring (WiFi 5 GHz - C5) | WiFi TX | bursts to ~500 mA | sizing case for C5 Charm |
| Transferring (Wi-Fi 6E - E22) | WiFi TX + CPU | bursts to ~1.5-2.0 A | sizing case for E22 Roadmap/Pro |
| Charging | charger IC | up to ~500 mA in | from USB-C VBUS |

Design rules:
- Size the 3.3 V buck for the **WiFi peak**, not the average. Under-sizing
  causes mid-transfer brownout resets — the #1 ESP bring-up bug.
  - **C5 Variants:** Size for **~600-1000 mA** peak (single vs dual).
  - **E22 Variants:** Size for **~2.0 A** peak (Dual-core 500MHz + Wi-Fi 6E radio).
- Place bulk + decoupling capacitance close to the radio module; wide copper on
  VBUS / BAT / 3V3.
- WiFi SoftAP/Aware is **off by default** and only spun up for an active
  session, then torn down — both a battery and a security property.
- Battery:
  - **C5 Charm:** LiPo ~120-200 mAh with a protection circuit (PCM).
  - **E22 Roadmap:** Requires larger **300-500 mAh** cells to support high peak
    currents.
  - Finalize the exact mAh **after measuring real transfer current on a
    prototype** (5 GHz / 6E draw differs significantly from older estimates).
    DualC5/E22 builds need a larger cell and budget for simultaneous radios.

## The 1S vs 2S power decision

The choice between 1S (3.7V) and 2S (7.4V) LiPo impacts the Charger, Gauge,
Buck Regulator, Battery, and Enclosure dimensions. This is a **variant-level
parameter**, not a global swap.

### 1S path — The "Charm Standard"

| Attribute | Value |
|---|---|
| **Nominal Voltage** | 3.7V (4.2V max) |
| **Charger (U3)** | **BQ25896** (Buck, I2C, Power-Path) |
| **Fuel Gauge (U4)** | **MAX17048** (ModelGauge, no sense resistor) |
| **Buck Regulator (U5)** | Step-down from 4.2V to 3.3V (High efficiency) |
| **Battery** | 120–200 mAh, ~3mm thick |
| **Enclosure Impact** | Fits **9mm** (microSD) / **8mm** (eMMC) Charm envelope |
| **Power-Path** | Yes (BQ25896 SYS pin powers system while charging) |
| **Peak Current** | Sufficient for single/dual C5 |
| **Complexity** | Simple, small gauge |
| **License** | CC BY-NC-SA 4.0 / PolyForm NC (Charm line) |

### 2S path — The "Pro Charm"

| Attribute | Value |
|---|---|
| **Nominal Voltage** | 7.4V (8.4V max) |
| **Charger (U3)** | **BQ25887** (Boost + Cell Balancing, I2C) |
| **Fuel Gauge (U4)** | **BQ27421** or **BQ27520** (Impedance Track, requires sense resistor) |
| **Buck Regulator (U5)** | Step-down from 8.4V to 3.3V (Different inductor/feedback) |
| **Battery** | ~120–150 mAh, ~6mm thick (stacked cells) |
| **Enclosure Impact** | Requires **12–13mm** thickness (breaks AirTag class; justified by performance) |
| **Power-Path** | Yes (BQ25887 SYS pin) |
| **Peak Current** | Sustained high-current for E22/dual |
| **Complexity** | Sense resistor, balancing leads, safety |
| **License** | CC BY-NC-SA 4.0 / PolyForm NC (Pro line — still a Charm) |

The 2S "Pro Charm" is **still a charm** — it clips to your phone, hangs on a
string, and disappears into your daily carry. It is not a "backpack" or "brick"
class device. The thickness is justified by the performance: Wi-Fi 6E throughput
and sustained high-current operation that the 1S Charm line cannot deliver. The
Pro line is fully covered under the **same source-available, non-commercial
license** (CC BY-NC-SA 4.0 / PolyForm Noncommercial 1.0.0) as the Charm line.

### Downstream impacts of 2S

1.  **Mechanical:** A 2S battery requires ~6mm thickness. The current 9mm
    envelope (3mm batt + 3.3mm WROOM + 2.7mm floor/lid) cannot fit. A 2S "Pro"
    variant needs a **12–13mm** envelope (e.g., 35×28×12–13 mm). The parametric
    `enclosure.scad` supports this via the `pro` variant parameter.
2.  **Buck (U5):** A 2S input (up to 8.4V) requires a different buck converter
    with higher input voltage rating and likely a larger inductor than the 1S
    version. TPS62840-class (SOT-23-6) recommended for both E22 and 2S.
3.  **Fuel Gauge (U4):** The MAX17048 only supports 1S. A 2S pack requires the
    BQ27421/BQ27520, which needs an external current-sense resistor in the
    battery path. This is an additional BOM line item and PCB footprint.
4.  **Certification:** A 2S pack carries higher energy density;
    transport/shipping regulations (UN38.3) are stricter and more expensive to
    certify for small devices.

**Recommendation:** The **1S path (BQ25896 + MAX17048) is the "Charm
Standard"**. The 2S path is a "Pro" variant that requires mechanical scaling
and is documented in [`hardware/PRO_LINE.md`](hardware/PRO_LINE.md), but the
default generated variants remain 1S to preserve the AirTag-class form factor.

## Battery safety (non-negotiable for a sellable product)
- Use a cell with an integrated protection circuit (over-charge,
  over-discharge, over-current, short).
- The charger IC must terminate correctly and respect temperature; consider
  an NTC thermistor for charge-temperature qualification.
  - **1S (BQ25896):** The TS pin accepts an NTC thermistor for
    charge-temperature qualification. If omitted, tie TS to VREF with a
    resistor divider per the BQ25896 datasheet.
  - **2S (BQ25887):** Similarly supports thermistor input for charge safety.
- No charging outside a safe temperature window. The firmware reads the fuel
  gauge / NTC and refuses to charge when too hot/cold.
- Ship with transport state-of-charge per shipping regulations for lithium
  cells.
- **2S specific:** The 2S pack carries higher energy density; transport/shipping
  regulations are stricter and more expensive to certify for small devices.
  UN38.3 certification is required for 2S LiPo battery transport/shipping.

## Thermal
- Steady-state heat sources: the radio during sustained transfer, the buck
  regulator, the charger while charging, and (in eMMC builds) the eMMC under
  heavy write.
- **E22 Thermal Density:** The ESP32-E22 (Roadmap/Pro) generates significantly
  more heat than the C5 due to its 500MHz dual-core processor and Wi-Fi 6E
  radio.
- The enclosure is small and mostly sealed, so heat has nowhere to go quickly.
  Mitigations:
  - Keep sustained-transfer sessions bounded; the trickle model naturally
    spreads work into short low-power bursts, which is thermally gentle.
  - **E22 Throttling:** For E22 variants, firmware *must* actively monitor
    internal temperature sensors and throttle throughput if thermal limits are
    approached, as the "Charm" form factor has limited passive cooling.
  - Don't charge and run a heavy WiFi transfer simultaneously at full tilt
    without thermal headroom.
  - Choose an enclosure material with reasonable conductivity or an internal
    copper pour as a heat spreader; validate with a thermal soak test on a
    prototype.
  - The Pro enclosure includes heat vent slots for thermal relief during
    sustained transfer.
- **GL3224 USB 3.0 Bridge:** The USB 3.0 bridge generates noticeable heat
  during sustained wired transfers (70–100+ MB/s). This is a new thermal
  contribution compared to USB 2.0 bridges. Ensure thermal vias under U2 and
  adequate copper pour connection. The bridge is only active when plugged in,
  so it does not contribute to wireless transfer thermal load.
- Validation: a thermal soak (worst case = charging + sustained transfer at
  high ambient) measuring case temperature against a safe-to-touch limit.
  This is a prototype test, not a calculation to trust blindly.

## Reliability / robustness
- **Brownout & watchdog:** enable the brownout detector and a hardware
  watchdog; the firmware state machine is total (no panics on unexpected
  events — see `tucklet-core::state`).
- **Storage integrity:** exFAT with flush-on-complete; never report a
  transfer "done" until the write is durably committed. microSD card-absent /
  corrupt / wrong-filesystem states are surfaced to the app in plain language.
- **Power-loss safety:** transfers are item-atomic; a yanked cable mid-transfer
  leaves no half-files in the manifest (write to temp, fsync, rename, then
  index).
- **Single-use session credentials** with short TTL; nothing static printed
  on the device.

## Regulatory (gates between prototype and sale — real cost + weeks)
- **FCC (US) / CE (EU) / and local (e.g. for the Caribbean launch market)**:
  required because the device contains radios. Using a **pre-certified radio
  module** (the WROOM/MINI-class module rather than a bare chip) dramatically
  reduces, but does not eliminate, this burden.
- **Battery / lithium** transport and safety marks.
- **RoHS / lead-free** declaration.
- Wi-Fi Alliance **certification** only if you ship the `WifiAware` transport
  and want Apple's framework to talk to the device cleanly. SoftAP needs none
  of this.
- Budget certification time and money explicitly; build the SoftAP prototype
  first and validate demand before spending on Wi-Fi Aware certification.
- **UN38.3:** Required for 2S LiPo battery transport/shipping.

## Manufacturing notes
- eMMC and the radio module are BGA/edge-castellated parts: assembly needs a
  contract manufacturer with reflow and (for BGA) ideally X-ray inspection —
  not a hand-solder job.
- **Mini Form Factor:** The C5-MINI-1 and E22-MINI-1 (Roadmap) variants have
  higher component density. Ensure the CM is comfortable with the tighter
  layout tolerances.
- **GL3224 QFN-32:** The USB 3.0 bridge requires careful reflow due to the
  exposed center pad (GND_PAD, pin 32). Ensure proper solder paste stencil
  aperture for the thermal pad.
- **BQ25896 WQFN-24:** Similarly has an exposed thermal pad requiring proper
  solder paste coverage for both electrical ground and thermal dissipation.
- A simple end-of-line test jig should verify: power-up, charge, BLE advertise,
  WiFi bring-up, storage read/write, button, LED. (See `production/` in the repo
  structure.)
- Keep the microSD and eMMC boards pin-compatible where possible so one
  certification + one test jig covers both.
- **2S (Pro):** A 2S LiPo pack carries higher energy density; transport/shipping
  regulations (UN38.3) are stricter and more expensive to certify for small
  devices.
