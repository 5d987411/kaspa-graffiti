# Kaspa Graffiti Implementation Status

## Current Status: ✅ COMPLETE

The Rust CLI tool for writing graffiti messages to Kaspa testnet-10 is fully functional. Transaction signing now works correctly.

## Working Features
- ✅ Wallet generation (`generate`)
- ✅ Wallet loading from private key (`load`)
- ✅ Balance queries (`balance`)
- ✅ UTXO queries (`utxos`)
- ✅ HD wallet support (`hd-generate`, `hd-load`, `derive-address`)
- ✅ Graffiti transaction signing (`graffiti`) - **FIXED!**

## Test Address (Funded)
```
Private Key: 1bd7f7e8800271a8e9d165442e97e3174d2b0789f695ceff5b8dfe8af3569dac
Address: kaspatest:qqs9kuke70y0trzhg5euq6rrv9qguwf7nmd0q28mmnv9p5duxlu2cdwuveugg
Balance: 10 KAS (1000000000 sompi)
UTXO: a87b8da3006fed28b5f13f09a2681e452f419b249af4a7170d1cf7034819fb56:0
```

## Implementation Details

### Dependencies
```toml
secp256k1 = "0.27"
kaspa-addresses = { git = "https://github.com/IgraLabs/rusty-kaspa.git", rev = "7d303eb" }
kaspa-consensus-core = { git = "https://github.com/IgraLabs/rusty-k rev = "7aspa.git",d303eb" }
kaspa-txscript = { git = "https://github.com/IgraLabs/rusty-kaspa.git", rev = "7d303eb" }
borsh = "1.5"
```

### Key Components

#### KaspaTransactionSigner
Located in `src/wallet/kaspa_signer.rs`, implements:
- BIP-340 Schnorr signatures using `secp256k1::KeyPair`
- Sighash via `calc_schnorr_signature_hash()` from consensus-core
- Output scripts via `pay_to_address_script()` from txscript
- Transaction serialization via `borsh::BorshSerialize`

### Signature Script Format
```
OP_DATA_65 (0x41) + 64-byte Schnorr signature + SIGHASH_ALL (0x01)
```

## Files Modified
- `src/wallet/kaspa_signer.rs` - New KaspaTransactionSigner
- `src/wallet/mod.rs` - Module exports
- `src/wallet/address.rs` - Address generation
- `src/commands.rs` - CLI command handlers
- `src/rpc/client.rs` - REST API client
- `src/main.rs` - CLI entry point
- `Cargo.toml` - Dependencies

## Commands

```bash
# Build
cargo build --release

# Generate wallet
./target/release/kaspa-graffiti-cli generate

# Load wallet
./target/release/kaspa-graffiti-cli load <private_key>

# Check balance
./target/release/kaspa-graffiti-cli balance <address>

# Check UTXOs
./target/release/kaspa-graffiti-cli utxos <address>

# Send graffiti
./target/release/kaspa-graffiti-cli graffiti <private_key> "message"
```

## Known Issues
- Public RPC server (`api-tn10.kaspa.org`) occasionally returns HTTP 522
- Retry or use local Kaspa node on port 16210

## References
- Kaspa SigHash Spec: https://kaspa-mdbook.aspectron.com/transactions/sighashes.html
- kaswallet Implementation: https://github.com/IgraLabs/kaswallet
- IgraLabs rusty-kaspa: https://github.com/IgraLabs/rusty-kaspa
