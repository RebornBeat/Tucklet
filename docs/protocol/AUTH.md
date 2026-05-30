# Tucklet auth contract (Ed25519 challenge-response)

This is the exact contract every client must follow. It is implemented and
cross-verified across the firmware (ed25519-dalek), desktop (shared
`tucklet-crypto`), iOS (CryptoKit), and Android (BouncyCastle).

## Keys and encodings
- **Algorithm:** Ed25519 (RFC 8032). Signatures are deterministic, so the same
  key + message always yields the same signature — which is how the stacks are
  cross-checked.
- **Public key:** 32 bytes, lowercase hex (64 chars).
- **Signature:** 64 bytes, lowercase hex (128 chars).
- **Nonce:** 32 raw bytes. The device sends it over BLE as
  `{"nonce":"<hex>"}`; the client **hex-decodes it and signs the raw bytes** —
  not the hex text. This is the single most common place to get it wrong.

## Flow
1. **Enrollment (one-time, requires the physical button press on the device).**
   The client generates an Ed25519 keypair, persists the private key in secure
   storage, and sends its public key in `PairRequest.phone_pubkey` (hex). The
   device stores it in the allow-list (`tucklet-core::allowlist`, NVS-persisted).

2. **Per-connection (silent, no button).**
   - The device generates a fresh random 32-byte `nonce` and pushes it on the
     SESSION characteristic.
   - The client hex-decodes the nonce, signs the **raw 32 bytes**, and sends the
     signature (hex) in `SessionRequest.challenge_signature`.
   - The device runs `verify_challenge(pubkey, nonce_bytes, signature_hex)`
     (`tucklet-crypto`): it must (a) find the pubkey in the allow-list and
     (b) verify the signature. Only then is a `SessionGrant` (Wi-Fi creds +
     bearer token) issued.

3. **Per data request.** The bearer token from the grant goes in the
   `X-Tucklet-Token` header on every `/v1` call.

## Secure storage per platform (the remaining device-confirm)
- **iOS:** CryptoKit key sealed in the Keychain (this-device-only).
- **Android:** BouncyCastle key sealed in EncryptedSharedPreferences (or
  Keystore/StrongBox where Ed25519 is supported).
- **Desktop:** `tucklet-crypto` key; persist via the OS keychain (currently a
  file stand-in under the config dir — `CONFIRM`).

## Reference vector (for new client implementations)
With `seed = 00 01 02 … 1f` and `nonce = 1f 1e … 00`:
- `pubkey = 03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8`
- `sig = a1a37d7f1f4418d2d31bc8699989836b8918b1849e801a59181ce6ddbd698dcd`
        `2acc49faa80677442d28e54a89f42ce35153c1bb60b68a3a06e980e52bec8508`

ed25519-dalek (firmware/desktop) and BouncyCastle (Android) both produce exactly
these values; a new client that matches them will interoperate.
