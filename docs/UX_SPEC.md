# UI/UX Specification (complete)

This is the full experience spec, folding in every insight from the original brief: plain-language state, per-app organization, round-trip metadata, user-controlled Temporary lifetimes, the "no cache vocabulary" rule, attachment virtualization, fast multi-item transfers with honest time estimates, trickle sync, battery/notifications, and an invisible-after-pairing hardware feel — all aimed at a non-technical user.

## Design principles
1. **Plain language, never jargon.** The words on screen are *On phone*, *On Tucklet*, *Temporary*. The word "cache" never appears anywhere in the product.
2. **The state is always visible.** Every item shows where it lives with a small, legible indicator — no mystery storage.
3. **The user decides, the device never surprises.** Nothing is auto-deleted without the user having chosen a Temporary lifetime. No hidden cleanup.
4. **It disappears.** After a one-time pairing there is no button-pressing, no "seeking", no maintenance. The charm is just there.
5. **Speed is honest.** Every transfer shows an estimate before it starts and a live ETA while it runs, computed from this unit's real link speed.
6. **Calm, warm, keepsake feel.** This is jewelry that holds memories, not a gadget. Soft, generous, unhurried — readable by someone who has never thought about storage.

## State vocabulary (the contract)
| Shown to user | Meaning | Indicator |
|---|---|---|
| **On phone** | Lives on the phone only | small phone glyph |
| **On Tucklet** | Lives on the device only | small charm glyph |
| **Temporary** | A copy on the phone that auto-removes when its timer ends | small clock glyph + remaining time |

`Temporary` lifetime is always the user's choice: **1 hour / 1 day / 1 week / Keep**. "Keep" promotes it to a permanent On-phone copy. The app enforces expiry; the canonical copy always remains On Tucklet.

## Information architecture
- **Home.** Device presence + battery + free/total space, the single most-common action ("Free up space" / "Get photos"), and a calm summary ("Everything's backed up" or "12 new photos waiting to back up").
- **Library.** Browse everything — *metadata and thumbnails only* (attachment virtualization). Full files move only on explicit action, which keeps the library instant and the battery untouched.
- **Organized by app, not folders.** Default buckets mirror where media lives: Camera, Screenshots, WhatsApp, Downloads, etc. A power-user "Folders" view exists but is never the default and never required.
- **Item / multi-select.** Tap an item for detail + state + origin; long-press to multi-select for batch actions.
- **Settings.** Paired phones (forget/revoke), default Temporary lifetime, trickle on/off + "only while charging", storage details, firmware.

## The transfer experience (the most important screen)
The brief's priority: move **many photos (10-30) or a few videos (2-3) fast**, and **tell the user how long it will take**.

Flow:
1. User selects items (or a whole app bucket) and chooses **Free up space** (Offload) or **Get a copy** (Load).
2. Before anything moves, a sheet shows: item count, total size, destination, and **estimated time** ("About 15s") computed by `tucklet-core::estimate` using this unit's `LinkProfile`. For Load, the Temporary picker appears here.
3. **Batch** runs in the foreground with a progress bar, count ("18 of 30"), and a **live ETA** that recomputes from measured throughput (`estimate::eta_seconds`) — so the number stays honest if the link slows.
4. On completion: a quiet confirmation and the freed space, with one-tap **Undo** for the last offload.

Two modes, surfaced plainly:
- **Now (Batch):** move it all immediately. Best when plugged in (wired ~20-40 MB/s) or for a quick set over Wi-Fi.
- **Automatically (Trickle):** the device backs up new photos in the background whenever it's near and idle/charging, so the big transfer never has to happen. The user sees "Backing up automatically — 142 of 300 done" rather than a wait.

Estimates by example (single C5, ~9 MB/s, close range — the realistic case):
- 30 photos (~4 MB each, 120 MB): **~15s**
- 3 videos (~500 MB each, 1.5 GB): **~3 min**
- Same 30 photos plugged in (wired bridge ~30 MB/s): **~5s**

These come straight from the tested estimator, including the per-file overhead that makes "many small photos" honest rather than optimistic.

## Round-trip restore
Each stored item remembers its `OriginMetadata` (app, collection, album, device). "Put back on phone" restores it into the same album/app, not an orphan folder — directly fixing the deep-folder pain that plagued older wireless drives.

## Battery, notifications, status
- Real battery % (hardware fuel gauge), charging state, and free space, always current via BLE `StatusReport`.
- Gentle, non-nagging notifications: "Tucklet is almost full", "Auto-backup finished (300 photos safe)", "Tucklet battery low". Never engagement-bait.

## The invisible hardware UX
- **First time only:** bring the charm near the phone, tap once to pair (the single physical button press, surfaced via the platform's accessory pairing flow).
- **Every time after:** zero presses. The phone recognizes it and connects silently and securely (cryptographic challenge-response against the allow-list). The button thereafter does only factory-reset (long hold).
- **Form:** small enough to hang from the phone on a string or sit on the back; it never blocks the charging port and is meant to be forgotten.

## Accessibility & inclusivity
- Large tap targets, high-contrast legible type, full screen-reader labels for every state indicator and the live ETA.
- Plain words over icons-only; every glyph has a text label.
- Works for someone who has never heard the word "sync".

## Cross-platform parity
- **Android:** the most seamless — programmatic Wi-Fi join, background trickle, system file-picker integration.
- **iOS:** AccessorySetupKit pairing, Wi-Fi Aware or SoftAP, Files-app integration via a File Provider extension; background trickle is partial (OS limits) and the UX states this honestly rather than pretending.
- **Desktop:** the device also appears as a plain fast USB drive when plugged in; wireless via the same local API.

The UI is identical in concept and vocabulary across all three; only the platform-specific connection plumbing differs underneath.
