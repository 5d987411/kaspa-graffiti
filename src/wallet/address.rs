use kaspa_addresses::{Address, Prefix, Version};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddressError {
    #[error("Invalid address format")]
    InvalidFormat,
    #[error("Invalid checksum")]
    BadChecksum,
    #[error("Unknown network")]
    UnknownNetwork,
}

#[derive(Debug, Clone, Copy)]
pub enum Network {
    Mainnet,
    Testnet10,
    Testnet11,
    Simnet,
}

impl Network {
    pub fn to_prefix(&self) -> Prefix {
        match self {
            Network::Mainnet => Prefix::Mainnet,
            Network::Testnet10 => Prefix::Testnet,
            Network::Testnet11 => Prefix::Testnet,
            Network::Simnet => Prefix::Simnet,
        }
    }

    pub fn from_name(name: &str) -> Result<Self, AddressError> {
        match name.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet-10" | "testnet10" => Ok(Network::Testnet10),
            "testnet-11" | "testnet11" => Ok(Network::Testnet11),
            "simnet" => Ok(Network::Simnet),
            _ => Err(AddressError::UnknownNetwork),
        }
    }
}

pub fn generate_address(public_key: &secp256k1::PublicKey, network: Network) -> String {
    let pubkey_bytes = public_key.serialize();
    // kaspa-addresses expects 32-byte x-only public key (no prefix byte)
    let xonly_pubkey = &pubkey_bytes[1..];

    let prefix = network.to_prefix();
    let address = Address::new(prefix, Version::PubKey, xonly_pubkey);
    address.to_string()
}

pub fn validate_address(address: &str, expected_network: Network) -> Result<bool, AddressError> {
    let prefix = expected_network.to_prefix();
    let addr = Address::try_from(address).map_err(|_| AddressError::InvalidFormat)?;
    if addr.prefix == prefix {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn extract_pubkey_hash_from_address(address: &str) -> Result<Vec<u8>, AddressError> {
    let addr = Address::try_from(address).map_err(|_| AddressError::InvalidFormat)?;
    Ok(addr.payload.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::KeyPair;

    #[test]
    fn test_address_generation_mainnet() {
        let keypair = KeyPair::new();
        let address = generate_address(keypair.public_key(), Network::Mainnet);
        assert!(
            address.starts_with("kaspa:"),
            "Expected kaspa: prefix, got: {}",
            address
        );
        assert!(address.len() > 35);
    }

    #[test]
    fn test_address_generation_testnet10() {
        let keypair = KeyPair::new();
        let address = generate_address(keypair.public_key(), Network::Testnet10);
        assert!(
            address.starts_with("kaspatest:"),
            "Expected kaspatest: prefix, got: {}",
            address
        );
    }

    #[test]
    fn test_known_address() {
        let known = "kaspa:qpauqsvk7yf9unexwmxsnmg547mhyga37csh0kj53q6xxgl24ydxjsgzthw5j";
        assert!(validate_address(known, Network::Mainnet).unwrap());
    }

    #[test]
    fn test_address_validation() {
        let keypair = KeyPair::new();
        let mainnet_address = generate_address(keypair.public_key(), Network::Mainnet);
        assert!(validate_address(&mainnet_address, Network::Mainnet).unwrap());

        let testnet_address = generate_address(keypair.public_key(), Network::Testnet10);
        assert!(validate_address(&testnet_address, Network::Testnet10).unwrap());
    }

    #[test]
    fn test_extract_pubkey_hash() {
        let keypair = KeyPair::new();
        let address = generate_address(keypair.public_key(), Network::Mainnet);
        let payload = extract_pubkey_hash_from_address(&address).unwrap();
        // PubKey version stores the 32-byte x-only public key
        assert_eq!(payload.len(), 32);
    }

    #[test]
    fn test_burn_address() {
        let burn_address = "kaspa:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkx9awp4e";
        assert!(validate_address(burn_address, Network::Mainnet).unwrap());
    }
}
