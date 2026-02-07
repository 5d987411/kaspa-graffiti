# Kaspa Graffiti Wallet

A Rust-based Kaspa wallet for testnet-10 with CLI and web UI interfaces.

## What This Wallet Does

### ✅ Core Features
- **Generate wallets** - Create new private keys and addresses
- **Load wallets** - Import wallets from private keys (hex format)
- **HD Wallets** - BIP32/BIP44 hierarchical deterministic wallets
- **Address Derivation** - Derive addresses from HD seeds
- **Check Balance** - Query address balance from Kaspa network
- **Get UTXOs** - List unspent transaction outputs
- **Transfer KAS** - Send Kaspa tokens to any address

### 🔐 Security
- Private keys stored locally (never sent to servers)
- Schnorr signatures (BIP-340 compliant)
- Local transaction signing

### 🌐 Network Support
- Testnet-10 (kaspatest:)
- Uses Kaspa public RPC API

## Quick Start

### CLI
```bash
# Build
cargo build --release

# Generate wallet
./target/release/kaspa-graffiti-cli generate

# Check balance
./target/release/kaspa-graffiti-cli balance kaspatest:qq...

# Transfer KAS
./target/release/kaspa-graffiti-cli transfer <private_key> <recipient> <amount>
./target/release/kaspa-graffiti-cli transfer <key> kaspatest:qq... 1.0
```

### Web UI
```bash
cd src-ui
node server.cjs
# Open http://localhost:8081
```

## Commands

| Command | Description |
|---------|-------------|
| `generate` | Generate new wallet |
| `load <key>` | Load wallet from private key |
| `hd-generate` | Generate HD wallet (seed) |
| `hd-load <seed>` | Load HD wallet |
| `derive-address <seed> <index>` | Derive single address |
| `derive-many <key> <count>` | Derive multiple addresses |
| `balance <address>` | Check balance |
| `utxos <address>` | Get UTXOs |
| `transfer <key> <addr> <amt>` | Send KAS (amt in KAS) |

## Web UI Features

- **Multiple wallets** - Load and switch between wallets
- **Wallet list** - Shows all loaded wallets with balances
- **One-click switch** - Click wallet to use for sending
- **TKAS display** - Shows TKAS for testnet addresses
- **Auto-refresh** - Balance updates after transfers
- **HD wallet** - Generate and derive addresses

## Project Structure

```
kaspa-graffiti/
├── src/
│   ├── commands.rs      # CLI commands
│   ├── wallet/          # Wallet, signing, HD
│   ├── rpc/            # Kaspa RPC client
│   └── graffiti/        # Graffiti message (disabled)
├── src-ui/             # Web interface
├── src-tauri/          # Tauri GUI (WIP)
└── tests/
```

## Build

```bash
# CLI only
cargo build --release

# With Tauri
cd src-tauri
cargo build --release
```

## Status

| Feature | Status |
|---------|--------|
| Wallet generation | ✅ Working |
| HD wallets | ✅ Working |
| Balance checking | ✅ Working |
| KAS transfers | ✅ Working |
| Multi-UTXO transactions | ✅ Working |
| Graffiti messages | ⚠️ Disabled |

## Technical Details

- **Signature**: BIP-340 Schnorr signatures
- **Transaction**: Version 0 (Kaspa requirement)
- **Fee**: Dynamic based on transaction mass
- **Min fee**: ~2000-7000 sompi depending on UTXOs

## License

MIT
