# Licensing

This document explains exactly what is licensed how, and — importantly — corrects a common misconception so you make this decision with eyes open.

## The misconception: "GNU lets others build but not sell"

This is **not true**, and getting it wrong could expose you legally or make your repo say something you don't mean.

- **GPL and every GNU license explicitly permit commercial sale.** Anyone may take GPL software, sell it, and even charge whatever they like. The GPL only requires they pass along the source and the same freedoms. "No selling" is fundamentally incompatible with the GPL and with the formal definition of "open source" (both the Free Software Foundation and the Open Source Initiative *require* that commercial use be allowed).
- So "others can build but not sell" is **not open source** at all. It is a **source-available, non-commercial** model. That's a legitimate choice — it's just not "GNU" or "open source," and you shouldn't label it as such.

## What Tucklet actually uses

Because your intent is *"others can study and build it, but I'm the one who sells it,"* Tucklet uses a non-commercial split:

| Layer | License | Why |
|---|---|---|
| **Firmware + app source** (`firmware/`, `software/`) | **PolyForm Noncommercial 1.0.0** | A clean, modern, plain-language source-available license. Grants everything *except* commercial use. Lets people learn from and contribute to your code without letting a competitor ship a clone for money. |
| **Hardware design files** (`hardware/`, `mechanical/`) | **CC BY-NC-SA 4.0** | The standard non-commercial license for design/creative files. Attribution required, non-commercial, share-alike (derivatives stay under the same terms). |

`LICENSE-SOFTWARE.txt` and `LICENSE-HARDWARE.txt` carry the canonical texts (full text fetched from the license authors — see the headers in those files for the source URL to paste in).

## The trade-off you are accepting

Non-commercial licensing is a real, defensible choice, but be aware:

1. **It deters some contributors.** Many open-source developers will not contribute to code they can never use commercially.
2. **It is not "open source."** Don't put "open source" on your packaging or store page — it's inaccurate and the OSI actively pushes back on misuse of the term. Say **"source-available"** instead.
3. **Enforcement is on you.** A license is only as strong as your willingness to enforce it. For a small seller, the practical protection is more about brand, support, and supply chain than litigation.

## If you change your mind later

Two common pivots:

- **Want maximum adoption and community?** Switch firmware/app to **Apache-2.0** or **MIT** and hardware to **CERN-OHL-S** (the hardware GPL). These *allow* selling — your moat becomes brand, distribution, and the assembled product, not the license. Most successful open hardware (Arduino, Adafruit, Pebble-era) works this way: the files are free, but people still buy the official polished unit.
- **Want a business but still some openness?** Keep the apps source-available non-commercial, but **fully close** the firmware and hardware. Perfectly normal for a commercial product.

You can relicense your *own* code at any time (you hold the copyright). You cannot retroactively relicense contributions others made under the old terms without their permission — so decide before you accept outside pull requests.

## What you must NOT do

- Do not call this "GNU" or "GPL." It isn't.
- Do not call it "open source" on commercial materials. It's "source-available."
- Do not mix in third-party GPL code into firmware you intend to keep non-commercial — GPL is "viral" and would force your whole firmware to become GPL (and thus commercially sellable by anyone). Audit every dependency's license. (The firmware's `Cargo.toml` lists dependency licenses for exactly this reason.)
