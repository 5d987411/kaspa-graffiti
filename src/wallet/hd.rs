use crate::wallet::{KeyPair, PrivateKey};
use hmac::{Hmac, Mac};
use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha512};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HdError {
    #[error("Invalid derivation path")]
    InvalidPath,
    #[error("Invalid child index")]
    InvalidIndex,
    #[error("Key derivation failed")]
    DerivationFailed,
    #[error("Hardened derivation requires private key")]
    HardenedRequiresPrivate,
}

type HmacSha512 = Hmac<Sha512>;

const HARDENED_OFFSET: u32 = 0x80000000;

#[derive(Debug, Clone)]
pub struct ExtendedKey {
    keypair: KeyPair,
    chain_code: [u8; 32],
    depth: u8,
    parent_fingerprint: [u8; 4],
    child_index: u32,
}

impl ExtendedKey {
    pub fn from_seed(seed: &[u8]) -> Result<Self, HdError> {
        let mut mac =
            HmacSha512::new_from_slice(b"Bitcoin seed").map_err(|_| HdError::DerivationFailed)?;
        mac.update(seed);
        let result = mac.finalize();
        let bytes = result.into_bytes();

        let (key_bytes, chain_code) = bytes.split_at(32);
        let secret_key =
            PrivateKey::from_slice(key_bytes).map_err(|_| HdError::DerivationFailed)?;

        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let keypair = KeyPair::from_secret_and_public(secret_key, public_key);

        let mut chain_code_arr = [0u8; 32];
        chain_code_arr.copy_from_slice(chain_code);

        Ok(Self {
            keypair,
            chain_code: chain_code_arr,
            depth: 0,
            parent_fingerprint: [0u8; 4],
            child_index: 0,
        })
    }

    pub fn from_keypair(keypair: KeyPair) -> Self {
        // Generate a deterministic chain code from the public key
        let pubkey_bytes = keypair.public_key_bytes();
        let hash = Sha512::digest(&pubkey_bytes);
        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&hash[..32]);

        Self {
            keypair,
            chain_code,
            depth: 0,
            parent_fingerprint: [0u8; 4],
            child_index: 0,
        }
    }

    pub fn derive_child(&self, index: u32) -> Result<Self, HdError> {
        let is_hardened = index >= HARDENED_OFFSET;

        let mut mac =
            HmacSha512::new_from_slice(&self.chain_code).map_err(|_| HdError::DerivationFailed)?;

        if is_hardened {
            mac.update(&[0u8]);
            mac.update(&self.keypair.secret_key().secret_bytes());
        } else {
            mac.update(&self.keypair.public_key_bytes());
        }

        mac.update(&index.to_be_bytes());
        let result = mac.finalize();
        let bytes = result.into_bytes();

        let (key_bytes, chain_code) = bytes.split_at(32);
        let mut child_key_bytes = [0u8; 32];
        child_key_bytes.copy_from_slice(key_bytes);

        let secp = Secp256k1::new();
        let parent_key_scalar = self.keypair.secret_key();
        let child_key_scalar =
            PrivateKey::from_slice(&child_key_bytes).map_err(|_| HdError::DerivationFailed)?;

        let mut new_secret_bytes = parent_key_scalar.secret_bytes();
        let child_bytes = child_key_scalar.secret_bytes();

        let mut carry: u16 = 0;
        for i in (0..32).rev() {
            let sum = new_secret_bytes[i] as u16 + child_bytes[i] as u16 + carry;
            new_secret_bytes[i] = (sum & 0xff) as u8;
            carry = sum >> 8;
        }

        // Attempt to create secret key - if invalid (0 or >= curve order), this index is invalid
        let new_secret = match PrivateKey::from_slice(&new_secret_bytes) {
            Ok(key) => key,
            Err(_) => return Err(HdError::InvalidIndex),
        };
        let new_public = PublicKey::from_secret_key(&secp, &new_secret);

        let keypair = KeyPair::from_secret_and_public(new_secret, new_public);

        let mut chain_code_arr = [0u8; 32];
        chain_code_arr.copy_from_slice(chain_code);

        let fingerprint = self.calculate_fingerprint();

        Ok(Self {
            keypair,
            chain_code: chain_code_arr,
            depth: self.depth + 1,
            parent_fingerprint: fingerprint,
            child_index: index,
        })
    }

    pub fn derive_path(&self, path: &str) -> Result<Self, HdError> {
        if !path.starts_with('m') {
            return Err(HdError::InvalidPath);
        }

        let path_part = &path[1..];
        if path_part.is_empty() {
            return Ok(self.clone());
        }

        let indices: Vec<u32> = path_part
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.ends_with('\'') || s.ends_with('h') {
                    let num: u32 = s
                        .trim_end_matches('\'')
                        .trim_end_matches('h')
                        .parse()
                        .map_err(|_| HdError::InvalidPath)?;
                    Ok(num + HARDENED_OFFSET)
                } else {
                    s.parse().map_err(|_| HdError::InvalidPath)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut current = self.clone();
        for index in indices {
            current = current.derive_child(index)?;
        }

        Ok(current)
    }

    pub fn derive_address_index(&self, index: u32) -> Result<Self, HdError> {
        let purpose = self.derive_child(44 + HARDENED_OFFSET)?;
        let coin_type = purpose.derive_child(111111 + HARDENED_OFFSET)?;
        let account = coin_type.derive_child(0 + HARDENED_OFFSET)?;
        let change = account.derive_child(0)?;
        change.derive_child(index)
    }

    pub fn derive_change_index(&self, index: u32) -> Result<Self, HdError> {
        let purpose = self.derive_child(44 + HARDENED_OFFSET)?;
        let coin_type = purpose.derive_child(111111 + HARDENED_OFFSET)?;
        let account = coin_type.derive_child(0 + HARDENED_OFFSET)?;
        let change = account.derive_child(1)?;
        change.derive_child(index)
    }

    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }

    pub fn chain_code(&self) -> &[u8; 32] {
        &self.chain_code
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn child_index(&self) -> u32 {
        self.child_index
    }

    fn calculate_fingerprint(&self) -> [u8; 4] {
        let pubkey_bytes = self.keypair.public_key_bytes();
        let hash = sha2::Sha512::digest(&pubkey_bytes);
        let mut fingerprint = [0u8; 4];
        fingerprint.copy_from_slice(&hash[..4]);
        fingerprint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let seed = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        let master = ExtendedKey::from_seed(&seed).unwrap();

        assert_eq!(master.depth(), 0);
        assert_eq!(master.child_index(), 0);

        let child0 = master.derive_child(0).unwrap();
        assert_eq!(child0.depth(), 1);
        assert_eq!(child0.child_index(), 0);

        let child1 = master.derive_child(1).unwrap();
        assert_eq!(child1.depth(), 1);
        assert_eq!(child1.child_index(), 1);

        assert_ne!(
            child0.keypair().to_hex(),
            child1.keypair().to_hex(),
            "Different indices should produce different keys"
        );
    }

    #[test]
    fn test_path_derivation() {
        let seed = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        let master = ExtendedKey::from_seed(&seed).unwrap();

        let child = master.derive_path("m/0/1").unwrap();
        assert_eq!(child.depth(), 2);
        assert_eq!(child.child_index(), 1);
    }

    #[test]
    fn test_address_index_derivation() {
        let seed = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        let master = ExtendedKey::from_seed(&seed).unwrap();

        let addr0 = master.derive_address_index(0).unwrap();
        let addr1 = master.derive_address_index(1).unwrap();

        assert_ne!(
            addr0.keypair().to_hex(),
            addr1.keypair().to_hex(),
            "Different address indices should produce different keys"
        );
    }

    #[test]
    fn test_deterministic_derivation() {
        let seed = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        let master1 = ExtendedKey::from_seed(&seed).unwrap();
        let master2 = ExtendedKey::from_seed(&seed).unwrap();

        let addr1 = master1.derive_address_index(5).unwrap();
        let addr2 = master2.derive_address_index(5).unwrap();

        assert_eq!(
            addr1.keypair().to_hex(),
            addr2.keypair().to_hex(),
            "Same seed and index should produce same key"
        );
    }
}
