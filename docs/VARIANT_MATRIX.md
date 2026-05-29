# Variant Matrix — every build configuration, and how one codebase serves all

Tucklet is not one device; it's a small product line from a single shared codebase. Three independent axes multiply into the full matrix. Firmware and apps adapt at runtime via the `DeviceCapabilities` descriptor (see `tucklet-proto`), so there is **one** firmware source and **one** app per platform — variants are configuration + capability negotiation, not forks.

## The three axes

| Axis | Options | Chosen by |
|---|---|---|
| Radio | `SingleC5` / `DualC5` | hardware build (compile-time feature) |
| Storage | `MicroSd` / `Emmc{capacity_gib}` | hardware build |
| Transport(s) | `SoftAp` (always) + optionally `WifiAware` + optionally `WiredUsbHs` | firmware feature flags + runtime |

2 radios x 2 storage families x transport sets = the full matrix below.

## The 8 headline configurations

| # | Radio | Storage | Wireless transport | Wired | Position |
|---|---|---|---|---|---|
| 1 | SingleC5 | microSD | SoftAp | USB-HS | **v1 baseline** — cheapest, universal, swappable |
| 2 | SingleC5 | microSD | SoftAp + WiFiAware | USB-HS | v1 + seamless iOS/Android |
| 3 | SingleC5 | eMMC | SoftAp | USB-HS | sealed/slim, fixed capacity |
| 4 | SingleC5 | eMMC | SoftAp + WiFiAware | USB-HS | sealed + seamless (premium) |
| 5 | DualC5 | microSD | SoftAp | USB-HS | speed-focused, swappable |
| 6 | DualC5 | microSD | SoftAp + WiFiAware | USB-HS | speed + seamless |
| 7 | DualC5 | eMMC | SoftAp | USB-HS | speed + sealed |
| 8 | DualC5 | eMMC | SoftAp + WiFiAware | USB-HS | top of line |

(microSD and eMMC each span the capacity sub-axis; eMMC capacities and BOM are in `hardware/common/VARIANTS.md`.)

## How firmware handles the matrix

Compile-time Cargo features select the hardware reality; everything else is runtime:

```
[features]
radio_single_c5 = []     # exactly one radio_* feature
radio_dual_c5   = []
storage_microsd = []     # exactly one storage_* feature
storage_emmc    = []
transport_softap = []    # always on
transport_wifi_aware = []# optional
transport_wired_usbhs = []# optional (the bridge IC is populated)
```

At boot the firmware assembles a `DeviceCapabilities` from its enabled features and advertises it. `tucklet-core::variant::usable_transports()` (unit-tested) then narrows that to what the connected client can use.

## How apps handle the matrix

The app never hard-codes a variant. On connect it reads `DeviceCapabilities` over BLE, then:

1. Calls the equivalent of `usable_transports(caps, platform)` to pick a transport (Wi-Fi Aware first if both sides have it, else SoftAP; wired when plugged in).
2. Loads the matching `LinkProfile` (`tucklet-core::link::profile_for`) so the transfer-time estimate reflects *this* unit's real speed.
3. Renders capacity, storage type, and battery from the live `StatusReport`.

So a user with a microSD SingleC5 and a user with a DualC5 eMMC run the identical app; it simply reports different speeds and capacities. The same is true across iOS, Android, and desktop because all three consume the same `tucklet-proto` types.

## Capability negotiation sequence

1. BLE connect + cryptographic auth (allow-list).
2. App reads `DeviceCapabilities` + `StatusReport`.
3. App resolves transport + link profile for the current platform and physical state (plugged in or not).
4. App shows accurate ETAs and storage states before any bytes move.
