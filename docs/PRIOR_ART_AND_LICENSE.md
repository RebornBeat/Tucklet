# Prior Art & License Validity

Answering, definitively: has anyone done this, what's the closest, and is our license valid?

## Has anyone done this? What's the closest?

The pieces all exist; the **combination + form factor + UX** does not. Closest products:

| Product | What it does | How it differs from Tucklet |
|---|---|---|
| **ThePhotoStick / "photo backup sticks"** | Wired USB stick + app, auto-backup photos, dedupe, no cloud, keepsake framing | **Wired**, plug-in each time; not wireless, not invisible, not proximity-automatic |
| **SanDisk Connect Wireless** (discontinued) | WiFi SoftAP + USB, app with auto photo backup, ~25 MB/s wired | Closest *architecture*. Discontinued. Bulky-ish, app was clunky, no proximity trickle, no charm form factor |
| **WD My Passport Wireless SSD** (discontinued) | One-button SD import, power bank, wireless app access | Closest *"one button, done"*. Pocket-brick, deep 6-level folders (a known pain), SD-card-centric, discontinued |
| **Home auto-backup / NAS appliances** | Pull photos automatically when you get home / charge | Closest *proximity auto-sync* idea, but stationary home devices, not a portable wearable |
| **FileHub-class travel routers** (RAVPower/NewQ) | Battery WiFi router + SD/USB sharing, dual-band ac | Deck-of-cards size, router-first, app-centric, not a charm, not auto |

**The unoccupied combination Tucklet occupies:**
1. **Charm / wearable form factor** that attaches to the phone and is meant to be forgotten.
2. **Automatic proximity trickle-sync** — backs up continuously so the big transfer never happens (time beats bandwidth).
3. **Clean, non-technical UX** — plain *On phone / On Tucklet / Temporary* states, per-app organization, round-trip restore, honest ETAs, and the word "cache" banished.
4. **Dual path** — invisible wireless for everyday + a genuinely fast wired bridge for bulk.

A telling signal: the two *best* prior products (SanDisk Connect, WD Wireless) were **discontinued**. The incumbents retreated, which both leaves the lane open and warns why — they were undone by slow transfers + clunky apps + awkward size. Tucklet's entire design targets exactly those failure modes.

## Is our license valid for this?

**Yes, unconditionally — and prior art does not affect it.**

- **Copyright licensing covers your own work.** You wrote the firmware, apps, and design files; you may license them however you like, including the source-available **non-commercial** terms already in the repo: **PolyForm Noncommercial 1.0.0** (code) + **CC BY-NC-SA 4.0** (hardware/design). The existence of other wireless drives does not restrict your right to license *your* implementation. You simply must not copy someone else's protected code, UI, or design files.
- **"Open to modify but not commercialize" is exactly what these licenses do.** (Reminder from `LICENSING.md`: this is *source-available, non-commercial*, NOT "open source"/GNU — GPL explicitly permits selling, so don't label it open source.)
- **Don't ingest copyleft (GPL) dependencies** into firmware/apps you intend to keep non-commercial; GPL would force commercial-sellability. Audit dependency licenses (`cargo deny check licenses`). The two crates built so far depend only on `serde` (MIT/Apache-2.0) — compatible.

## Patents (a separate, low-but-nonzero concern)
- Copyright (above) and patents are different things. Broad concepts like "wireless photo backup" or "dedupe on import" may be covered by utility patents held by larger companies.
- For a small independent seller this risk is **low in practice** (you're not a lawsuit target at small volume) but **nonzero**, and it grows with scale.
- Practical posture: don't copy a specific competitor's protected mechanism; keep your implementation your own; if/when you scale or raise money, get a real freedom-to-operate review from an IP attorney. Not a blocker for building now.

## Bottom line
No one has shipped this specific thing. Your closest ancestors either retreated from the market or are wired/clunky/brick-sized. Your license is valid and matches your intent. The novelty that matters is execution — form factor, proximity trickle-sync, and a genuinely humane UX.
