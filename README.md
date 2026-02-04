# kaspa-graffiti

# KaspaGraffiti

Post messages to the Kaspa blockchain (testnet-10).

## Overview

KaspaGraffiti is a Tauri application that allows you to:
- Generate or load a Kaspa wallet
- Check your balance on testnet-10
- Post text messages to the Kaspa blockchain
- Messages are stored on-chain using script data

## Architecture

```
kaspa-graffiti/
├── src/                      # Rust core library
│   ├── wallet/              # Key generation, addresses, transactions
│   ├── rpc/                 # Kaspa RPC client
│   ├── graffiti/            # Message encoding/decoding
│   └── commands.rs          # Tauri commands
├── src-tauri/               # Tauri application
├── src-ui/                  # React + TypeScript frontend
└── Cargo.toml
```

## Building

### Prerequisites

- Rust 1.70+
- Node.js 18+
- A running Kaspa testnet-10 node (default: 127.0.0.1:16210)

### Build Steps

```bash
# Install dependencies
cd kaspa-graffiti

# Build Rust backend
cargo build

# Install frontend dependencies
cd src-ui
npm install

# Build frontend
npm run build

# Run development
cargo tauri dev
```

## Usage

### Generate a New Wallet

1. Click "Generate New Wallet"
2. Save your private key securely
3. Your address will be displayed

### Load Existing Wallet

1. Enter your private key (hex format)
2. Click "Load Wallet"

### Post a Message

1. Make sure you have a wallet loaded
2. Go to the "Compose" tab
3. Enter your message (max 500 characters)
4. Optionally adjust the RPC URL and fee rate
5. Click "Post to Blockchain"

## Message Format

Messages are encoded as JSON and stored in the transaction's script data:

```json
{
  "version": 1,
  "timestamp": 1234567890,
  "content": "Your message here",
  "mimetype": "text/plain",
  "nonce": 0
}
```

## Networks

Currently configured for Kaspa testnet-10 (default RPC: 127.0.0.1:16210)

## Security Notes

- Private keys are displayed once during wallet generation - save them securely
- Messages are stored openly on the blockchain - no encryption
- Use testnet-10 only - never send real funds

## Dependencies

### Rust
- secp256k1 - Elliptic curve cryptography
- sha2 - SHA-256 hashing
- ripemd - RIPEMD-160 hashing
- bs58 - Base58 encoding
- tokio - Async runtime
- serde - Serialization

### TypeScript
- React 18
- @tauri-apps/api
- Vite

## License

MIT
