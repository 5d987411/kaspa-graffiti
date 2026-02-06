use std::collections::BTreeMap;
use std::iter::once;

use borsh::BorshSerialize;
use kaspa_addresses::Address;
use kaspa_consensus_core::hashing::sighash::{
    calc_schnorr_signature_hash, SigHashReusedValuesUnsync,
};
use kaspa_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use kaspa_consensus_core::tx::{
    MutableTransaction, ScriptPublicKey, Transaction, TransactionId, TransactionInput,
    TransactionOutpoint, TransactionOutput, UtxoEntry,
};
use kaspa_txscript::pay_to_address_script;
use secp256k1::{Message, Secp256k1};

const SIG_HASH_ALL_U8: u8 = 0x01;

#[derive(Debug, Clone)]
pub struct KaspaSignedTransaction {
    pub tx_hex: String,
    pub tx_id: String,
}

impl KaspaSignedTransaction {
    pub fn hex(&self) -> &str {
        &self.tx_hex
    }

    pub fn id(&self) -> &str {
        &self.tx_id
    }
}

pub struct KaspaTransactionSigner {
    transaction: Transaction,
    utxos: Vec<UtxoEntry>,
}

impl KaspaTransactionSigner {
    pub fn new() -> Self {
        Self {
            transaction: Transaction::new(
                1,
                Vec::new(),
                Vec::new(),
                0,
                Default::default(),
                0,
                Vec::new(),
            ),
            utxos: Vec::new(),
        }
    }

    pub fn add_input(
        &mut self,
        txid: &str,
        vout: u32,
        amount: u64,
        script_pubkey: &[u8],
    ) -> Result<(), String> {
        let txid_bytes = hex::decode(txid).map_err(|e| format!("Invalid txid: {}", e))?;
        let txid_obj = TransactionId::from_bytes(
            txid_bytes
                .try_into()
                .map_err(|_| "Invalid txid length, expected 32 bytes")?,
        );

        let outpoint = TransactionOutpoint {
            transaction_id: txid_obj,
            index: vout,
        };

        let script_public_key = ScriptPublicKey::new(0, script_pubkey.to_vec().into());
        let utxo = UtxoEntry::new(amount, script_public_key.clone(), 0, false);

        self.utxos.push(utxo.clone());

        let input = TransactionInput {
            previous_outpoint: outpoint,
            signature_script: Vec::new(),
            sequence: 0,
            sig_op_count: 1,
        };

        self.transaction.inputs.push(input);

        Ok(())
    }

    pub fn add_output(&mut self, address: &str, amount: u64) -> Result<(), String> {
        let address = Address::try_from(address).map_err(|e| format!("Invalid address: {}", e))?;
        let script_pubkey = pay_to_address_script(&address);

        let output = TransactionOutput {
            value: amount,
            script_public_key: script_pubkey,
        };

        self.transaction.outputs.push(output);

        Ok(())
    }

    pub fn set_payload(&mut self, payload: &[u8]) {
        self.transaction.payload = payload.to_vec();
    }

    pub fn sign(&mut self, private_key: &[u8]) -> Result<KaspaSignedTransaction, String> {
        let secp = Secp256k1::new();
        let keypair = secp256k1::KeyPair::from_seckey_slice(&secp, private_key)
            .map_err(|e| format!("Invalid private key: {}", e))?;

        let (xonly_pubkey, _) = keypair.x_only_public_key();
        let pubkey_bytes: [u8; 32] = xonly_pubkey.serialize();

        eprintln!("DEBUG: X-only public key: {}", hex::encode(&pubkey_bytes));

        // Create signable transaction with UTXO entries
        let mut signable_tx =
            MutableTransaction::with_entries(self.transaction.clone(), self.utxos.clone());

        // Create map from script pubkey to keypair (same as kaswallet)
        let mut map: BTreeMap<Vec<u8>, secp256k1::KeyPair> = BTreeMap::new();
        let script_pub_key_script: Vec<u8> = once(0x20)
            .chain(pubkey_bytes.iter().copied())
            .chain(once(0xac))
            .collect();
        map.insert(script_pub_key_script, keypair);

        let reused_values = SigHashReusedValuesUnsync::new();

        // Sign each input (same as kaswallet's sign_with_multiple)
        for i in 0..signable_tx.tx.inputs.len() {
            let script = signable_tx.entries[i]
                .as_ref()
                .unwrap()
                .script_public_key
                .script();

            if let Some(schnorr_key) = map.get(script) {
                // Calculate sighash using Kaspa's official function
                let sig_hash = calc_schnorr_signature_hash(
                    &signable_tx.as_verifiable(),
                    i,
                    SIG_HASH_ALL,
                    &reused_values,
                );

                eprintln!("DEBUG: Sighash {}: {}", i, hex::encode(sig_hash.as_bytes()));

                // Create message from sighash
                let msg = Message::from_slice(&sig_hash.as_bytes())
                    .map_err(|e| format!("Failed to create message: {}", e))?;

                // Sign with Schnorr using Secp256k1 context
                let sig: [u8; 64] = *secp.sign_schnorr_no_aux_rand(&msg, &schnorr_key).as_ref();

                // Build signature script: OP_DATA_65 + 64-byte sig + SIG_HASH_ALL
                // This is exactly what kaswallet does
                let signature_script: Vec<u8> = once(65u8)
                    .chain(sig.iter().copied())
                    .chain([SIG_HASH_ALL.to_u8()])
                    .collect();

                eprintln!(
                    "DEBUG: Signature script ({} bytes): {}",
                    signature_script.len(),
                    hex::encode(&signature_script)
                );

                signable_tx.tx.inputs[i].signature_script = signature_script;
                signable_tx.tx.inputs[i].sig_op_count = 1;
            } else {
                return Err(format!("No key found for input {}", i));
            }
        }

        // Serialize transaction using borsh
        let mut serialized = Vec::new();
        borsh::BorshSerialize::serialize(&signable_tx.tx, &mut serialized)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let tx_hex = hex::encode(serialized.clone());

        // Calculate transaction ID by finalizing the transaction
        let mut tx_final = signable_tx.tx.clone();
        tx_final.finalize();
        let tx_id = tx_final.id();
        let tx_id_hex = hex::encode(tx_id.as_bytes());

        eprintln!("DEBUG: Signed tx ID: {}", tx_id_hex);
        eprintln!("DEBUG: Signed tx hex length: {}", tx_hex.len());

        Ok(KaspaSignedTransaction {
            tx_hex,
            tx_id: tx_id_hex,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_creation() {
        let signer = KaspaTransactionSigner::new();
        assert!(signer.transaction.inputs.is_empty());
        assert!(signer.transaction.outputs.is_empty());
    }
}
