# Variant Matrix — every build configuration, and how one codebase serves all

Tucklet is not one device; it's a small product line from a single shared codebase. Four independent axes multiply into the full matrix. Firmware and apps adapt at runtime via the `DeviceCapabilities` descriptor (see `tucklet-proto`), so there is **one** firmware source and **one** app per platform — variants are configuration + capability negotiation, not forks.

## The four axes

| Axis | Options | Chosen by |
|---|---|---|
| **Chip** | `C5` (Production) / `E22` (Roadmap) | hardware build (compile-time feature) |
| **Form Factor** | `WROOM` (Standard) / `MINI` (Compact) | hardware build |
| **Radio Count** | `Single` / `Dual` | hardware build |
| **Storage** | `MicroSd` / `Emmc{capacity_gib}` | hardware build |
| **Transport(s)** | `SoftAp` (always) + optionally `WifiAware` + optionally `WiredUsbHs` | firmware feature flags + runtime |

Theoretical maximum: 2 Chips x 2 Forms x 2 Counts x 2 Storage = 16 Variants.
**Actual Generated Variants:** 12 (See "Charm Strategy Exclusion" below).

## Charm Strategy Exclusion
To maintain the "Charm" product identity (invisible, robust, AirTag-class), we enforce a strict size constraint.
*   **Excluded:** `E22-WROOM` variants. The WROOM/M.2 module is oversized (22x30mm) and requires an external antenna (IPEX), which violates the sealed/integrated design philosophy.
*   **Included:** `E22-MINI` variants. We anticipate a future Mini module that fits the Charm form factor. These are marked as **Roadmap** items.

## The 12 generated configurations

### Tier 1: The "Charm" (Standard C5)
The production-ready baseline using the ESP32-C5-WROOM-1 module.

| # | Radio | Storage | Wireless | Wired | Position |
|---|---|---|---|---|---|
| 1 | SingleC5 | microSD | SoftAp | USB-HS | **Baseline** — cheapest, swappable |
| 2 | SingleC5 | eMMC | SoftAp | USB-HS | Slim, sealed, fixed capacity |
| 3 | DualC5 | microSD | SoftAp | USB-HS | Speed-focused, swappable |
| 4 | DualC5 | eMMC | SoftAp | USB-HS | Speed + sealed |

### Tier 2: The "Nano" (Mini C5)
The ultra-compact production line using the ESP32-C5-MINI-1 module.

| # | Radio | Storage | Wireless | Wired | Position |
|---|---|---|---|---|---|
| 5 | SingleC5 | microSD | SoftAp | USB-HS | Ultra-compact, swappable |
| 6 | SingleC5 | eMMC | SoftAp | USB-HS | Smallest sealed charm |
| 7 | DualC5 | microSD | SoftAp | USB-HS | Experimental speed in Nano form |
| 8 | DualC5 | eMMC | SoftAp | USB-HS | Top-tier Nano |

### Tier 3: The "Super-Charm" (Roadmap E22)
High-performance variants dependent on the future release of an ESP32-E22-MINI-1 module.

| # | Radio | Storage | Wireless | Wired | Position |
|---|---|---|---|---|---|
| 9 | SingleE22 | microSD | SoftAp | USB-HS | High-speed roadmap |
| 10 | SingleE22 | eMMC | SoftAp | USB-HS | High-speed sealed roadmap |
| 11 | DualE22 | microSD | SoftAp | USB-HS | Extreme performance roadmap |
| 12 | DualE22 | eMMC | SoftAp | USB-HS | Ultimate speed roadmap |

*(Note: microSD and eMMC each span the capacity sub-axis; eMMC capacities and BOM are in `hardware/common/VARIANTS.md`.)*

## How firmware handles the matrix

Compile-time Cargo features select the hardware reality; everything else is runtime:

```
[features]
# Chip selection
chip_c5 = []
chip_e22 = []

# Form Factor
form_wroom = []
form_mini  = []

# Radio Count (derived or explicit)
radio_single = []
radio_dual   = []

# Storage
storage_microsd = []
storage_emmc    = []

# Transport
transport_softap = []        # always on
transport_wifi_aware = []    # optional
transport_wired_usbhs = []   # optional (the bridge IC is populated)
```

At boot the firmware assembles a `DeviceCapabilities` from its enabled features and advertises it. `tucklet-core::variant::usable_transports()` (unit-tested) then narrows that to what the connected client can use.

## How apps handle the matrix

The app never hard-codes a variant. On connect it reads `DeviceCapabilities` over BLE, then:

1. Calls the equivalent of `usable_transports(caps, platform)` to pick a transport (Wi-Fi Aware first if both sides have it, else SoftAP; wired when plugged in).
2. Loads the matching `LinkProfile` (`tucklet-core::link::profile_for`) so the transfer-time estimate reflects *this* unit's real speed.
3. Renders capacity, storage type, and battery from the live `StatusReport`.

So a user with a microSD SingleC5 Mini and a user with a DualC5 eMMC Standard run the identical app; it simply reports different speeds and capacities. The same is true across iOS, Android, and desktop because all three consume the same `tucklet-proto` types.

## Capability negotiation sequence

1. BLE connect + cryptographic auth (allow-list).
2. App reads `DeviceCapabilities` + `StatusReport`.
3. App resolves transport + link profile for the current platform and physical state (plugged in or not).
4. App shows accurate ETAs and storage states before any bytes move.
