use crate::wallet::{KeyPair, Network, generate_address, KaspaTransactionSigner};
use crate::rpc::RpcClient;
use crate::graffiti::{GraffitiMessage, PayloadEncoder};
use crate::{KaspaGraffitiError, Result};
use secp256k1::Secp256k1;

pub async fn generate_wallet() -> Result<WalletInfo> {
    let keypair = KeyPair::new();
    let address = crate::wallet::generate_address(keypair.public_key(), Network::Testnet10);

    Ok(WalletInfo {
        private_key: keypair.to_hex(),
        public_key: keypair.public_key_hex(),
        address,
        network: "testnet-10".to_string(),
    })
}

pub async fn load_wallet(private_key: &str) -> Result<WalletInfo> {
    let keypair = KeyPair::from_hex(private_key)
        .map_err(|_| KaspaGraffitiError::InvalidPrivateKey)?;
    let address = crate::wallet::generate_address(keypair.public_key(), Network::Testnet10);

    Ok(WalletInfo {
        private_key: keypair.to_hex(),
        public_key: keypair.public_key_hex(),
        address,
        network: "testnet-10".to_string(),
    })
}

pub async fn validate_address(address: &str) -> bool {
    crate::wallet::validate_address(address, Network::Testnet10).unwrap_or(false)
}

pub async fn generate_hd_wallet() -> Result<HDWalletInfo> {
    use rand::RngCore;

    let mut seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);

    let extended_key = crate::wallet::ExtendedKey::from_seed(&seed)
        .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?;
    
    let address0 = extended_key.derive_address_index(0)
        .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?;
    
    let address = crate::wallet::generate_address(address0.keypair().public_key(), Network::Testnet10);
    
    Ok(HDWalletInfo {
        seed: hex::encode(seed),
        address,
        network: "testnet-10".to_string(),
    })
}

pub async fn load_hd_wallet(seed_hex: &str) -> Result<HDWalletInfo> {
    let seed = hex::decode(seed_hex)
        .map_err(|_| KaspaGraffitiError::InvalidPrivateKey)?;
    if seed.len() != 32 {
        return Err(KaspaGraffitiError::InvalidPrivateKey);
    }
    
    let extended_key = crate::wallet::ExtendedKey::from_seed(seed.as_slice())
        .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?;
    
    let address0 = extended_key.derive_address_index(0)
        .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?;
    
    let address = crate::wallet::generate_address(address0.keypair().public_key(), Network::Testnet10);
    
    Ok(HDWalletInfo {
        seed: hex::encode(seed),
        address,
        network: "testnet-10".to_string(),
    })
}

pub async fn derive_address_from_seed(seed_hex: &str, index: u32, is_change: bool) -> Result<DerivedAddressInfo> {
    let seed = hex::decode(seed_hex)
        .map_err(|_| KaspaGraffitiError::InvalidPrivateKey)?;
    if seed.len() != 32 {
        return Err(KaspaGraffitiError::InvalidPrivateKey);
    }
    
    let extended_key = crate::wallet::ExtendedKey::from_seed(seed.as_slice())
        .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?;
    
    let derived = if is_change {
        extended_key.derive_change_index(index)
            .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?
    } else {
        extended_key.derive_address_index(index)
            .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?
    };
    
    let keypair = derived.keypair();
    let address = crate::wallet::generate_address(keypair.public_key(), Network::Testnet10);
    
    Ok(DerivedAddressInfo {
        address,
        index,
        is_change,
        private_key: keypair.to_hex(),
        public_key: keypair.public_key_hex(),
    })
}

pub async fn derive_many_addresses(seed_hex: &str, count: u32, is_change: bool) -> Result<Vec<DerivedAddressInfo>> {
    let seed = hex::decode(seed_hex)
        .map_err(|_| KaspaGraffitiError::InvalidPrivateKey)?;
    if seed.len() != 32 {
        return Err(KaspaGraffitiError::InvalidPrivateKey);
    }
    
    let extended_key = crate::wallet::ExtendedKey::from_seed(seed.as_slice())
        .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?;
    
    let mut addresses = Vec::new();
    for i in 0..count {
        let derived = if is_change {
            extended_key.derive_change_index(i)
                .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?
        } else {
            extended_key.derive_address_index(i)
                .map_err(|e| KaspaGraffitiError::Wallet(e.to_string()))?
        };
        
        let keypair = derived.keypair();
        let address = crate::wallet::generate_address(keypair.public_key(), Network::Testnet10);
        
        addresses.push(DerivedAddressInfo {
            address,
            index: i,
            is_change,
            private_key: keypair.to_hex(),
            public_key: keypair.public_key_hex(),
        });
    }
    
    Ok(addresses)
}

#[derive(serde::Serialize)]
pub struct SendResult {
    pub txid: String,
    pub fee: u64,
    pub change: u64,
    pub address: String,
}

#[derive(serde::Serialize)]
pub struct UtxoInfo {
    pub txid: String,
    pub vout: u32,
    pub amount: u64,
    pub script_pubkey: String,
}

pub async fn get_balance(
    address: &str,
    rpc_url: Option<&str>,
) -> Result<BalanceInfo> {
    let client = RpcClient::new(rpc_url);

    let response = client.get_balance_by_address(address).await
        .map_err(|e| KaspaGraffitiError::Rpc(e.to_string()))?;

    Ok(BalanceInfo {
        balance: response.balance,
        address: address.to_string(),
    })
}

pub async fn get_utxos(
    address: &str,
    rpc_url: Option<&str>,
) -> Result<Vec<UtxoInfo>> {
    let client = RpcClient::new(rpc_url);

    let response = client.get_utxos_by_address(address).await
        .map_err(|e| KaspaGraffitiError::Rpc(e.to_string()))?;

    let utxos: Vec<UtxoInfo> = response.entries.into_iter().map(|e| UtxoInfo {
        txid: e.outpoint.transaction_id,
        vout: e.outpoint.index,
        amount: e.utxo_entry.amount,
        script_pubkey: e.utxo_entry.script_public_key.script,
    }).collect();

    Ok(utxos)
}

pub async fn send_graffiti(
    private_key: &str,
    message: &str,
    mimetype: Option<&str>,
    rpc_url: Option<&str>,
    fee_rate: u64,
) -> Result<SendResult> {
    let private_bytes = hex::decode(private_key)
        .map_err(|_| KaspaGraffitiError::InvalidPrivateKey)?;
    if private_bytes.len() != 32 {
        return Err(KaspaGraffitiError::InvalidPrivateKey);
    }
    let private_key_array: [u8; 32] = private_bytes.try_into()
        .map_err(|_| KaspaGraffitiError::InvalidPrivateKey)?;

    let secp = Secp256k1::new();
    let keypair = secp256k1::KeyPair::from_seckey_slice(&secp, &private_key_array)
        .map_err(|_| KaspaGraffitiError::InvalidPrivateKey)?;
    let (xonly_pubkey, _) = keypair.x_only_public_key();
    let xonly_bytes: [u8; 32] = xonly_pubkey.serialize();

    // Create address directly using kaspa-addresses API
    use kaspa_addresses::{Address, Prefix, Version};
    let prefix = Network::Testnet10.to_prefix();
    let address = Address::new(prefix, Version::PubKey, &xonly_bytes);
    let address = address.to_string();

    let client = RpcClient::new(rpc_url);

    let utxos_response = client.get_utxos_by_addresses(vec![address.clone()]).await
        .map_err(|e| KaspaGraffitiError::Rpc(e.to_string()))?;

    if utxos_response.entries.is_empty() {
        return Err(KaspaGraffitiError::NoUtxos);
    }

    let message_bytes = message.as_bytes().to_vec();
    if message_bytes.len() > 100 {
        return Err(KaspaGraffitiError::Encoding(
            format!("Message too long: {} bytes (max: 100)", message_bytes.len())
        ));
    }

    let mut signer = KaspaTransactionSigner::new();

    let mut total_input: u64 = 0;
    for utxo in &utxos_response.entries {
        let script_pubkey_hex = &utxo.utxo_entry.script_public_key.script;
        let script_pubkey: Vec<u8> = hex::decode(script_pubkey_hex)
            .map_err(|e| KaspaGraffitiError::Encoding(e.to_string()))?;

        signer.add_input(
            &utxo.outpoint.transaction_id,
            utxo.outpoint.index,
            utxo.utxo_entry.amount,
            &script_pubkey,
        ).map_err(|e| KaspaGraffitiError::Transaction(e.to_string()))?;
        total_input += utxo.utxo_entry.amount;
    }

    let estimated_fee = 1000;
    let change_amount = total_input.saturating_sub(estimated_fee);

    if change_amount < 1000 {
        return Err(KaspaGraffitiError::InsufficientBalance(total_input, estimated_fee));
    }

    signer.add_output(&address, change_amount)
        .map_err(|e| KaspaGraffitiError::Transaction(e.to_string()))?;

    signer.set_payload(&message_bytes);

    let signed_tx = signer.sign(&private_key_array)
        .map_err(|e| KaspaGraffitiError::Transaction(e.to_string()))?;

    let submit_response = client.submit_transaction_hex(signed_tx.hex()).await
        .map_err(|e| KaspaGraffitiError::Rpc(e.to_string()))?;

    Ok(SendResult {
        txid: submit_response.transaction_id,
        fee: estimated_fee,
        change: change_amount,
        address,
    })
}

#[derive(serde::Serialize)]
pub struct WalletInfo {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
    pub network: String,
}

#[derive(serde::Serialize)]
pub struct BalanceInfo {
    pub balance: u64,
    pub address: String,
}

#[derive(serde::Serialize)]
pub struct HDWalletInfo {
    pub seed: String,
    pub address: String,
    pub network: String,
}

#[derive(serde::Serialize)]
pub struct DerivedAddressInfo {
    pub address: String,
    pub index: u32,
    pub is_change: bool,
    pub private_key: String,
    pub public_key: String,
}
