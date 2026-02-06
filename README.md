# Kaspa Graffiti CLI - Implementation Complete

## Status: ✅ WORKING

The Rust CLI tool for writing graffiti messages to Kaspa testnet-10 is now fully functional.

## What Was Fixed

### Core Signing Issues Resolved

1. **Transaction Serialization**
   - Added `borsh` dependency for proper transaction serialization
   - Transaction now serializes correctly for Kaspa node acceptance

2. **Signature Implementation**
   - Changed from `Keypair` to `KeyPair` (secp256k1 naming)
   - Used `secp.sign_schnorr_no_aux_rand()` for BIP-340 compliant signatures
   - Fixed sighash calculation using `calc_schnorr_signature_hash()`

3. **Dependencies Updated**
   ```
   borsh = "1.5"
   kaspa-txscript = { git = "https://github.com/IgraLabs/rusty-kaspa.git", rev = "7d303eb" }
   ```

4. **Key Files Modified**
   - `src/wallet/kaspa_signer.rs` - New KaspaTransactionSigner using IgraLabs/kaswallet pattern
   - `src/rpc/client.rs` - Fixed REST API structures
   - `src/commands.rs` - Updated to use KeyPair and proper address creation
   - `Cargo.toml` - Added borsh and kaspa-txscript dependencies

## Test Address (Funded)

```
Private Key: 1bd7f7e8800271a8e9d165442e97e3174d2b0789f695ceff5b8dfe8af3569dac
Address: kaspatest:qqs9kuke70y0trzhg5euq6rrv9qguwf7nmd0q28mmnv9p5duxlu2cdwuveugg
Balance: 10 KAS (1000000000 sompi)
UTXO: a87b8da3006fed28b5f13f09a2681e452f419b249af4a7170d1cf7034819fb56:0
```

## Commands Available

```bash
# Wallet Management
kaspa-graffiti-cli generate              # Generate new wallet
kaspa-graffiti-cli load <key>           # Load from private key
kaspa-graffiti-cli hd-generate           # Generate HD wallet
kaspa-graffiti-cli hd-load <seed>       # Load HD wallet

# Address Derivation
kaspa-graffiti-cli derive <seed> <index> [change]
kaspa-graffiti-cli derive-many <seed> <count>

# Network Queries
kaspa-graffiti-cli balance <address>    # Get balance
kaspa-graffiti-cli utxos <address>     # Get UTXOs

# Graffiti (Working!)
kaspa-graffiti-cli graffiti <key> <message>
```

## Technical Details

### Signature Script Format (BIP-340)
```
OP_DATA_65 (0x41) + 64-byte Schnorr signature + SIGHASH_ALL (0x01)
```

### Sighash Algorithm
- Uses Kaspa's `calc_schnorr_signature_hash()` from consensus-core
- Implements BIP-143-style sighash for Kaspa

### Transaction Structure
- Version: 1
- Inputs: 1 (spending the funded UTXO)
- Outputs: 1 (change to sender)
- Payload: Graffiti message bytes
- Subnetwork: Native (0x00..00)

## Known Issues

- Public RPC server (`api-tn10.kaspa.org`) occasionally returns HTTP 522
- Solution: Retry or use local Kaspa node on port 16210

## References

- Kaspa SigHash Spec: https://kaspa-mdbook.aspectron.com/transactions/sighashes.html
- kaswallet Implementation: https://github.com/IgraLabs/kaswallet
- IgraLabs rusty-kaspa: https://github.com/IgraLabs/rusty-kaspa

## Build & Run

```bash
cd /home/cliff/kaspa-graffiti
cargo build --release

# Test graffiti
./target/release/kaspa-graffiti-cli graffiti \
  1bd7f7e8800271a8e9d165442e97e3174d2b0789f695ceff5b8dfe8af3569dac \
  "Hello Kaspa!"
```
