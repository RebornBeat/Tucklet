// Reference vector generator: fixed seed + nonce -> pubkey + signature (hex).
// Used to cross-check the Android (BouncyCastle) and iOS (CryptoKit) signers,
// since RFC 8032 Ed25519 is deterministic.
use ed25519_dalek::{Signer, SigningKey};
fn main() {
    let seed: [u8; 32] = core::array::from_fn(|i| i as u8); // 0,1,...,31
    let nonce: [u8; 32] = core::array::from_fn(|i| (31 - i) as u8); // 31,30,...,0
    let sk = SigningKey::from_bytes(&seed);
    println!("seed={}", hex::encode(seed));
    println!("nonce={}", hex::encode(nonce));
    println!("pubkey={}", hex::encode(sk.verifying_key().to_bytes()));
    println!("sig={}", hex::encode(sk.sign(&nonce).to_bytes()));
}
