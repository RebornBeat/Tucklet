// tucklet-crypto
// The one security primitive, shared and standards-based (RFC 8032 Ed25519):
//   * a client (phone/desktop) holds an `Identity` (Ed25519 keypair),
//   * it enrolls its public key (hex) with the device at pairing,
//   * each connection the device issues a random 32-byte nonce,
//   * the client signs the RAW nonce bytes and returns the signature (hex),
//   * the device verifies the signature against the enrolled public key.
//
// The exact contract that all clients (firmware verify, desktop, iOS CryptoKit,
// Android BouncyCastle) MUST agree on:
//   - pubkey  = 32-byte Ed25519 public key, lowercase hex (64 chars)
//   - nonce   = the RAW 32 bytes (the device sends them hex-encoded on BLE; the
//               client hex-decodes first, then signs the bytes — NOT the hex text)
//   - sig     = 64-byte Ed25519 signature, lowercase hex (128 chars)
//
// This crate is host-testable (used by the desktop client and the unit tests);
// the firmware's no_std build uses the same `ed25519_dalek` primitives.
//
// License: PolyForm Noncommercial 1.0.0

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::string::String;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

pub const NONCE_LEN: usize = 32;
pub const PUBKEY_LEN: usize = 32;
pub const SIG_LEN: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub enum CryptoError {
    BadHex,
    BadPubkeyLen,
    BadSecretLen,
    BadSigLen,
    InvalidPubkey,
}

/// A client signing identity (private + public Ed25519 key).
pub struct Identity {
    signing: SigningKey,
}

impl Identity {
    /// Generate a fresh identity from a cryptographically secure RNG.
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        let mut rng = rand_core::OsRng;
        Identity {
            signing: SigningKey::generate(&mut rng),
        }
    }

    /// Reconstruct from a stored 32-byte secret seed.
    pub fn from_secret_bytes(seed: &[u8]) -> Result<Self, CryptoError> {
        let arr: [u8; 32] = seed.try_into().map_err(|_| CryptoError::BadSecretLen)?;
        Ok(Identity {
            signing: SigningKey::from_bytes(&arr),
        })
    }

    /// The 32-byte secret seed, for secure persistence (keychain / NVS / file).
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing.to_bytes()
    }

    /// Lowercase-hex public key to enroll with the device at pairing.
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing.verifying_key().to_bytes())
    }

    /// Sign a challenge: input is the RAW nonce bytes (already hex-decoded from
    /// what the device sent). Returns the signature as lowercase hex.
    pub fn sign_nonce(&self, nonce: &[u8]) -> String {
        let sig: Signature = self.signing.sign(nonce);
        hex::encode(sig.to_bytes())
    }
}

/// Decode a hex-encoded 32-byte Ed25519 public key.
pub fn decode_public_key(pubkey_hex: &str) -> Result<VerifyingKey, CryptoError> {
    let bytes = hex::decode(pubkey_hex).map_err(|_| CryptoError::BadHex)?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::BadPubkeyLen)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| CryptoError::InvalidPubkey)
}

/// The device side: verify that `signature_hex` is a valid Ed25519 signature
/// over `nonce` for the given public key. (The caller also checks the key is in
/// the allow-list before granting a session — see tucklet-core::allowlist.)
pub fn verify_challenge(
    pubkey_hex: &str,
    nonce: &[u8],
    signature_hex: &str,
) -> Result<bool, CryptoError> {
    let vk = decode_public_key(pubkey_hex)?;
    let sig_bytes = hex::decode(signature_hex).map_err(|_| CryptoError::BadHex)?;
    let sig_arr: [u8; 64] = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::BadSigLen)?;
    let sig = Signature::from_bytes(&sig_arr);
    Ok(vk.verify(nonce, &sig).is_ok())
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    fn nonce() -> [u8; 32] {
        // A deterministic "device nonce" for tests.
        let mut n = [0u8; 32];
        for (i, b) in n.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(1);
        }
        n
    }

    #[test]
    fn sign_then_device_verifies() {
        let id = Identity::generate();
        let pk = id.public_key_hex();
        let n = nonce();
        let sig = id.sign_nonce(&n);
        // Exactly the device's check (same primitives as firmware auth.rs).
        assert!(verify_challenge(&pk, &n, &sig).unwrap());
    }

    #[test]
    fn tampered_nonce_fails() {
        let id = Identity::generate();
        let pk = id.public_key_hex();
        let n = nonce();
        let sig = id.sign_nonce(&n);
        let mut bad = n;
        bad[0] ^= 0xFF;
        assert!(!verify_challenge(&pk, &bad, &sig).unwrap());
    }

    #[test]
    fn wrong_key_fails() {
        let signer = Identity::generate();
        let other = Identity::generate();
        let n = nonce();
        let sig = signer.sign_nonce(&n);
        // A different enrolled key must not verify this signature.
        assert!(!verify_challenge(&other.public_key_hex(), &n, &sig).unwrap());
    }

    #[test]
    fn secret_round_trips_and_keeps_identity() {
        let id = Identity::generate();
        let secret = id.secret_bytes();
        let pk = id.public_key_hex();
        let restored = Identity::from_secret_bytes(&secret).unwrap();
        assert_eq!(restored.public_key_hex(), pk);
        // A signature from the restored identity still verifies.
        let n = nonce();
        assert!(verify_challenge(&pk, &n, &restored.sign_nonce(&n)).unwrap());
    }

    #[test]
    fn pubkey_and_sig_have_expected_hex_lengths() {
        let id = Identity::generate();
        assert_eq!(id.public_key_hex().len(), PUBKEY_LEN * 2);
        assert_eq!(id.sign_nonce(&nonce()).len(), SIG_LEN * 2);
    }

    #[test]
    fn malformed_inputs_error_cleanly() {
        assert_eq!(decode_public_key("zz").unwrap_err(), CryptoError::BadHex);
        assert_eq!(decode_public_key("00").unwrap_err(), CryptoError::BadPubkeyLen);
        let id = Identity::generate();
        // good key, bad signature hex
        assert!(matches!(
            verify_challenge(&id.public_key_hex(), &nonce(), "00"),
            Ok(false) | Err(CryptoError::BadSigLen)
        ));
    }
}
