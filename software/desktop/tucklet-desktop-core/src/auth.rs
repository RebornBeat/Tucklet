// auth.rs
// The desktop machine's signing identity — the real, shared Ed25519 primitive
// (tucklet-crypto). On first run a keypair is created; its public key is enrolled
// with the device at pairing, and each session nonce is signed with the private
// key. The device verifies with the same crate.
//
// Persistence here writes the 32-byte secret to a file under the OS config dir.
// CONFIRM: move this to the OS keychain (macOS Keychain / Windows DPAPI /
// libsecret) before shipping; the file is a portable stand-in, not the final
// at-rest protection.
//
// License: PolyForm Noncommercial 1.0.0

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tucklet_crypto::Identity as KeyIdentity;

pub struct Identity {
    inner: KeyIdentity,
}

impl Identity {
    /// Load the per-machine identity, creating + persisting one on first run.
    pub fn load_or_create() -> std::io::Result<Self> {
        let path = key_path();
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(inner) = KeyIdentity::from_secret_bytes(&bytes) {
                return Ok(Identity { inner });
            }
        }
        let inner = KeyIdentity::generate();
        persist_secret(&path, &inner.secret_bytes())?;
        Ok(Identity { inner })
    }

    /// Load from an explicit path (used by tests).
    pub fn load_or_create_at(path: &Path) -> std::io::Result<Self> {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(inner) = KeyIdentity::from_secret_bytes(&bytes) {
                return Ok(Identity { inner });
            }
        }
        let inner = KeyIdentity::generate();
        persist_secret(path, &inner.secret_bytes())?;
        Ok(Identity { inner })
    }

    /// Hex public key, enrolled with the device at pairing.
    pub fn public_key_hex(&self) -> String {
        self.inner.public_key_hex()
    }

    /// Sign a session challenge. `nonce` is the RAW bytes (hex-decode what the
    /// device sent on the SESSION characteristic first). Returns the hex sig.
    pub fn sign_challenge(&self, nonce: &[u8]) -> String {
        self.inner.sign_nonce(nonce)
    }
}

fn key_path() -> PathBuf {
    // ~/.config/tucklet/identity.key (or platform equivalent via env).
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("tucklet").join("identity.key")
}

fn persist_secret(path: &Path, secret: &[u8; 32]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut f = fs::File::create(path)?;
    // Best-effort owner-only perms on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = f.metadata()?.permissions();
        perms.set_mode(0o600);
        let _ = f.set_permissions(perms);
    }
    f.write_all(secret)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucklet_crypto::verify_challenge;

    #[test]
    fn identity_persists_and_signs_verifiably() {
        let dir = std::env::temp_dir().join(format!("tucklet-id-{}", std::process::id()));
        let path = dir.join("identity.key");
        let _ = fs::remove_file(&path);

        let id = Identity::load_or_create_at(&path).unwrap();
        let pk = id.public_key_hex();

        // Simulate the device handshake: a 32-byte nonce, signed, then verified
        // exactly as the firmware does (same shared crate).
        let nonce = [9u8; 32];
        let sig = id.sign_challenge(&nonce);
        assert!(verify_challenge(&pk, &nonce, &sig).unwrap());

        // Reloading yields the same identity (persistence works).
        let again = Identity::load_or_create_at(&path).unwrap();
        assert_eq!(again.public_key_hex(), pk);
        let _ = fs::remove_dir_all(&dir);
    }
}
