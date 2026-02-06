pub mod wallet;
pub mod rpc;
pub mod graffiti;
pub mod commands;

pub use wallet::{KeyPair};
pub use rpc::RpcClient;
pub use graffiti::{GraffitiMessage, PayloadEncoder};
pub use commands::{WalletInfo, BalanceInfo, UtxoInfo, SendResult, HDWalletInfo, DerivedAddressInfo};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KaspaGraffitiError {
    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("No UTXOs available")]
    NoUtxos,

    #[error("Insufficient balance: have {0}, need {1}")]
    InsufficientBalance(u64, u64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, KaspaGraffitiError>;
