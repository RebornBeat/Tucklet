//! Pairing + per-connection authentication.
//!
//! Trust model (see docs/FINAL_REVIEW.md §3):
//!   * Enrollment requires the physical button press — the only way a new phone
//!     enters the allow-list.
//!   * Daily connection is silent: the device issues a random nonce, the phone
//!     signs it with its private key, and the device verifies the signature
//!     against the enrolled Ed25519 public key. No button, not spoofable.
//!
//! The allow-list itself is the host-tested [`tucklet_core::allowlist::AllowList`];
//! this module adds NVS persistence and the crypto.

use anyhow::{anyhow, Result};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use tucklet_core::allowlist::{AllowList, TrustedPhone};

const NVS_NAMESPACE: &str = "tucklet";
const NVS_KEY_ALLOWLIST: &str = "allowlist";
const NVS_KEY_DEVICE_SK: &str = "device_sk"; // device's own Ed25519 secret (for mutual auth)

/// Persisted pairing state, backed by NVS.
pub struct AuthStore {
    nvs: EspNvs<NvsDefault>,
    list: AllowList,
}

impl AuthStore {
    /// Load the allow-list from NVS (empty on first boot).
    pub fn load(nvs: EspNvs<NvsDefault>) -> Self {
        let mut store = Self { nvs, list: AllowList::new() };
        if let Ok(Some(json)) = store.read_blob(NVS_KEY_ALLOWLIST) {
            if let Ok(entries) = serde_json::from_slice::<Vec<(String, String)>>(&json) {
                for (pubkey, name) in entries {
                    store.list.enroll(pubkey, name);
                }
            }
        }
        store
    }

    fn read_blob(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Two-step: get length, then read. esp-idf-svc returns the slice on read.
        let mut buf = vec![0u8; 1024];
        match self.nvs.get_blob(key, &mut buf)? {
            Some(slice) => Ok(Some(slice.to_vec())),
            None => Ok(None),
        }
    }

    fn persist(&mut self) -> Result<()> {
        let entries: Vec<(String, String)> = self
            .list
            .phones()
            .iter()
            .map(|p| (p.pubkey.clone(), p.name.clone()))
            .collect();
        let json = serde_json::to_vec(&entries)?;
        self.nvs.set_blob(NVS_KEY_ALLOWLIST, &json)?;
        Ok(())
    }

    /// Enroll a phone (called only after the physical button press). Returns
    /// true if newly added. Persists immediately.
    pub fn enroll(&mut self, pubkey: String, name: String) -> Result<bool> {
        // Reject malformed keys up front.
        decode_verifying_key(&pubkey)?;
        let added = self.list.enroll(pubkey, name);
        self.persist()?;
        Ok(added)
    }

    pub fn is_known(&self, pubkey: &str) -> bool {
        self.list.contains(pubkey)
    }

    pub fn phones(&self) -> &[TrustedPhone] {
        self.list.phones()
    }

    /// Revoke a single phone; persists.
    pub fn revoke(&mut self, pubkey: &str) -> Result<bool> {
        let removed = self.list.revoke(pubkey);
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    /// Factory reset: clear the whole allow-list (long button hold). Persists.
    pub fn factory_reset(&mut self) -> Result<()> {
        self.list.clear();
        self.persist()
    }

    /// Verify that `signature_hex` is a valid Ed25519 signature over `nonce`,
    /// produced by an enrolled phone identified by `pubkey_hex`.
    ///
    /// Returns Ok(true) only if (a) the phone is in the allow-list, and (b) the
    /// signature verifies — both required for a silent daily connection. The
    /// cryptography itself is the shared, host-tested `tucklet_crypto`.
    pub fn verify_challenge(
        &self,
        pubkey_hex: &str,
        nonce: &[u8],
        signature_hex: &str,
    ) -> Result<bool> {
        if !self.is_known(pubkey_hex) {
            return Ok(false);
        }
        tucklet_crypto::verify_challenge(pubkey_hex, nonce, signature_hex)
            .map_err(|e| anyhow!("verify failed: {e:?}"))
    }
}

fn decode_verifying_key(pubkey_hex: &str) -> Result<()> {
    // Validate the key is a well-formed Ed25519 public key (used at enrollment).
    tucklet_crypto::decode_public_key(pubkey_hex)
        .map(|_| ())
        .map_err(|e| anyhow!("bad pubkey: {e:?}"))
}

impl AuthStore {
    /// Get-or-create the device's own Ed25519 keypair (for mutual auth: the
    /// phone can pin the device's public key at pairing time). The 32-byte
    /// `seed` is taken from the hardware RNG by the caller on first boot only;
    /// thereafter the persisted secret is reused. Returns the public key hex.
    pub fn device_pubkey_hex(&mut self, seed: [u8; 32]) -> Result<String> {
        use ed25519_dalek::SigningKey;
        if let Ok(Some(sk_bytes)) = self.read_blob(NVS_KEY_DEVICE_SK) {
            if let Ok(arr) = <[u8; 32]>::try_from(sk_bytes.as_slice()) {
                let sk = SigningKey::from_bytes(&arr);
                return Ok(hex::encode(sk.verifying_key().to_bytes()));
            }
        }
        // First boot: persist a fresh secret derived from the RNG seed.
        let sk = SigningKey::from_bytes(&seed);
        self.nvs.set_blob(NVS_KEY_DEVICE_SK, &seed)?;
        Ok(hex::encode(sk.verifying_key().to_bytes()))
    }
}
