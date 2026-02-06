use hex;
use rand::rngs::OsRng;
use rand::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyError {
    #[error("Invalid private key length")]
    InvalidLength,
    #[error("Invalid private key format")]
    InvalidFormat,
    #[error("Failed to parse key")]
    ParseError,
}

pub type PrivateKey = SecretKey;
pub type PublicKeyCompressed = PublicKey;

#[derive(Debug, Clone)]
pub struct KeyPair {
    secret_key: PrivateKey,
    public_key: PublicKeyCompressed,
}

impl KeyPair {
    pub fn new() -> Self {
        let secp = Secp256k1::new();

        let mut rng = OsRng;
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);

        let secret_key =
            PrivateKey::from_slice(&secret_bytes).expect("Failed to create secret key");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Self {
            secret_key,
            public_key,
        }
    }

    pub fn from_hex(hex_key: &str) -> Result<Self, KeyError> {
        let key_bytes = hex::decode(hex_key).map_err(|_| KeyError::InvalidFormat)?;

        if key_bytes.len() != 32 {
            return Err(KeyError::InvalidLength);
        }

        let secret_key = PrivateKey::from_slice(&key_bytes).map_err(|_| KeyError::ParseError)?;

        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.secret_key.secret_bytes())
    }

    pub fn public_key_bytes(&self) -> [u8; 33] {
        self.public_key.serialize()
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.serialize())
    }

    pub fn secret_key(&self) -> &PrivateKey {
        &self.secret_key
    }

    pub fn public_key(&self) -> &PublicKeyCompressed {
        &self.public_key
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.secret_key.secret_bytes()
    }

    pub fn from_secret_and_public(secret_key: PrivateKey, public_key: PublicKeyCompressed) -> Self {
        Self {
            secret_key,
            public_key,
        }
    }
}

impl Default for KeyPair {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let keypair = KeyPair::new();
        let hex = keypair.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_key_from_hex() {
        let keypair = KeyPair::new();
        let hex = keypair.to_hex();

        let recovered = KeyPair::from_hex(&hex).unwrap();
        assert_eq!(recovered.to_hex(), hex);
    }

    #[test]
    fn test_invalid_hex() {
        assert!(KeyPair::from_hex("invalid").is_err());
        assert!(KeyPair::from_hex("123").is_err());
        assert!(KeyPair::from_hex("").is_err());
    }
}
