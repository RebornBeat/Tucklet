# Power, Thermal & Production-Readiness Engineering

Everything that has to be true for the device to be safe, reliable, and manufacturable — not just functional on a bench.

## Power budget
The charm must sip power at rest (so it lasts weeks and feels "always there") and tolerate WiFi current spikes during transfer without browning out.

| State | Dominant draw | Approx current | Notes |
|---|---|---|---|
| Asleep | RTC + leakage | ~10-50 uA | radios off, deep sleep, button/timer wake |
| Advertising (BLE low-duty) | BLE TX bursts | ~tens-hundreds uA avg | rotating address; weeks on a small cell |
| Connected (BLE only) | BLE link | ~3-8 mA | STATUS notifications |
| Transferring (WiFi 5 GHz) | WiFi TX | bursts to ~500 mA | the sizing case |
| Charging | charger IC | up to ~500 mA in | from USB-C VBUS |

Design rules:
- Size the 3.3 V buck for the **WiFi peak (~500 mA bursts)**, not the average. Under-sizing causes mid-transfer brownout resets — the #1 ESP bring-up bug.
- Place bulk + decoupling capacitance close to the radio module; wide copper on VBUS / BAT / 3V3.
- WiFi SoftAP/Aware is **off by default** and only spun up for an active session, then torn down — both a battery and a security property.
- Battery: LiPo ~120-200 mAh with a protection circuit (PCM). Finalize the exact mAh **after measuring real 5 GHz transfer current on a C5 prototype** (5 GHz draw differs from the older S3 estimates). DualC5 builds need a larger cell and budget for two simultaneous radios.

## Battery safety (non-negotiable for a sellable product)
- Use a cell with an integrated protection circuit (over-charge, over-discharge, over-current, short).
- The charger IC must terminate correctly and respect temperature; consider an NTC thermistor for charge-temperature qualification.
- No charging outside a safe temperature window. The firmware reads the fuel gauge / NTC and refuses to charge when too hot/cold.
- Ship with transport state-of-charge per shipping regulations for lithium cells.

## Thermal
- Steady-state heat sources: the radio during sustained transfer, the buck regulator, the charger while charging, and (in eMMC builds) the eMMC under heavy write.
- The enclosure is small and mostly sealed, so heat has nowhere to go quickly. Mitigations:
  - Keep sustained-transfer sessions bounded; the trickle model naturally spreads work into short low-power bursts, which is thermally gentle.
  - Don't charge and run a heavy WiFi transfer simultaneously at full tilt without thermal headroom; firmware can throttle transfer rate if a temperature threshold is crossed.
  - Choose an enclosure material with reasonable conductivity or an internal copper pour as a heat spreader; validate with a thermal soak test on a prototype.
- Validation: a thermal soak (worst case = charging + sustained transfer at high ambient) measuring case temperature against a safe-to-touch limit. This is a prototype test, not a calculation to trust blindly.

## Reliability / robustness
- **Brownout & watchdog:** enable the brownout detector and a hardware watchdog; the firmware state machine is total (no panics on unexpected events — see `tucklet-core::state`).
- **Storage integrity:** exFAT with flush-on-complete; never report a transfer "done" until the write is durably committed. microSD card-absent / corrupt / wrong-filesystem states are surfaced to the app in plain language.
- **Power-loss safety:** transfers are item-atomic; a yanked cable mid-transfer leaves no half-files in the manifest (write to temp, fsync, rename, then index).
- **Single-use session credentials** with short TTL; nothing static printed on the device.

## Regulatory (gates between prototype and sale — real cost + weeks)
- **FCC (US) / CE (EU) / and local (e.g. for the Caribbean launch market)**: required because the device contains radios. Using a **pre-certified radio module** (the WROOM-class module rather than a bare chip) dramatically reduces, but does not eliminate, this burden.
- **Battery / lithium** transport and safety marks.
- **RoHS / lead-free** declaration.
- Wi-Fi Alliance **certification** only if you ship the `WifiAware` transport and want Apple's framework to talk to the device cleanly. SoftAP needs none of this.
- Budget certification time and money explicitly; build the SoftAP prototype first and validate demand before spending on Wi-Fi Aware certification.

## Manufacturing notes
- eMMC and the radio module are BGA/edge-castellated parts: assembly needs a contract manufacturer with reflow and (for BGA) ideally X-ray inspection — not a hand-solder job.
- A simple end-of-line test jig should verify: power-up, charge, BLE advertise, WiFi bring-up, storage read/write, button, LED. (See `production/` in the repo structure.)
- Keep the microSD and eMMC boards pin-compatible where possible so one certification + one test jig covers both.

## What still requires a prototype to finalize (honest list)
- Exact battery mAh (after measuring real 5 GHz transfer current).
- Thermal soak limits and any transfer throttling threshold.
- Antenna tuning, and (for DualC5) two-antenna isolation.
- exFAT-over-SDIO timing and the USB-bridge storage-mux arbitration.
These are bring-up tasks, not design unknowns; the architecture above is settled.
