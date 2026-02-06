## Files Modified

### Core Implementation
- `src/wallet/kaspa_signer.rs` - NEW: KaspaTransactionSigner using kaswallet pattern
- `src/wallet/mod.rs` - Added exports for new signer
- `src/wallet/address.rs` - Fixed for x-only pubkey handling
- `src/commands.rs` - Fixed KeyPair usage and address creation
- `src/rpc/client.rs` - Fixed REST API response structures
- `src/main.rs` - Fixed function calls and output format
- `Cargo.toml` - Added borsh and kaspa-txscript dependencies

## Changes Summary

### 1. kaspa_signer.rs (New)
Implements Kaspa transaction signing using:
- `calc_schnorr_signature_hash()` for BIP-143 sighash
- `pay_to_address_script()` for output script creation
- `borsh::BorshSerialize` for transaction serialization
- BIP-340 Schnorr signatures with SIGHASH_ALL

### 2. Cargo.toml
Added:
- `borsh = "1.5"` for serialization
- `kaspa-txscript` for script utilities

### 3. Key Fixes
- `Keypair` → `KeyPair` (secp256k1 naming)
- `Message::from_digest_slice()` → `Message::from_slice()`
- `sign_schnorr()` → `secp.sign_schnorr_no_aux_rand()`
- `to_script_public_key()` → `pay_to_address_script()`

## Test Results

| Command | Status |
|---------|--------|
| Balance check | ✅ Working |
| UTXO fetching | ✅ Working |
| Transaction build | ✅ Working |
| Transaction sign | ✅ Working |
| Submit to RPC | ⚠️ 522 timeout (server issue) |
