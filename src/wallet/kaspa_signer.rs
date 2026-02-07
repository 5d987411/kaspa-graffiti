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
use serde::Serialize;

const SIG_HASH_ALL_U8: u8 = 0x01;

const MASS_PER_TX_BYTE: u64 = 1;
const MASS_PER_SCRIPT_PUB_KEY_BYTE: u64 = 10;
const MASS_PER_SIG_OP: u64 = 1000;

fn compute_transaction_mass(tx: &Transaction) -> u64 {
    let mut size: u64 = 0;
    size += 2;
    size += 8;
    for input in &tx.inputs {
        size += 32;
        size += 4;
        size += 8;
        size += input.signature_script.len() as u64;
        size += 8;
    }
    size += 8;
    for output in &tx.outputs {
        size += 8;
        size += 2;
        size += 8;
        size += output.script_public_key.script().len() as u64;
    }
    size += 8;
    size += 20;
    size += 8;
    size += tx.payload.len() as u64;

    let compute_mass_for_size = size * MASS_PER_TX_BYTE;
    let total_script_pub_key_size: u64 = tx
        .outputs
        .iter()
        .map(|output| 2 + output.script_public_key.script().len() as u64)
        .sum();
    let total_script_pub_key_mass = total_script_pub_key_size * MASS_PER_SCRIPT_PUB_KEY_BYTE;
    let total_sigops: u64 = tx
        .inputs
        .iter()
        .map(|input| input.sig_op_count as u64)
        .sum();
    let total_sigops_mass = total_sigops * MASS_PER_SIG_OP;

    compute_mass_for_size + total_script_pub_key_mass + total_sigops_mass
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTransactionInput {
    #[serde(rename = "previousOutpoint")]
    pub previous_outpoint: JsonOutPoint,
    #[serde(rename = "signatureScript")]
    pub signature_script: String,
    pub sequence: u64,
    #[serde(rename = "sigOpCount")]
    pub sig_op_count: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonOutPoint {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonScriptPublicKey {
    pub version: u16,
    #[serde(rename = "scriptPublicKey")]
    pub script: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTransactionOutput {
    #[serde(rename = "amount")]
    pub amount: u64,
    #[serde(rename = "scriptPublicKey")]
    pub script_public_key: JsonScriptPublicKey,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTransaction {
    pub version: u32,
    pub inputs: Vec<JsonTransactionInput>,
    pub outputs: Vec<JsonTransactionOutput>,
    #[serde(rename = "lockTime")]
    pub lock_time: u64,
    #[serde(rename = "subnetworkId")]
    pub subnetwork_id: String,
    pub gas: u64,
    #[serde(rename = "payload")]
    pub payload: String,
    pub mass: u64,
}

#[derive(Debug, Clone)]
pub struct KaspaSignedTransaction {
    pub tx_hex: String,
    pub tx_id: String,
    pub json_tx: JsonTransaction,
}

impl KaspaSignedTransaction {
    pub fn hex(&self) -> &str {
        &self.tx_hex
    }

    pub fn id(&self) -> &str {
        &self.tx_id
    }

    pub fn json(&self) -> &JsonTransaction {
        &self.json_tx
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
                0,
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

        // Create map from script pubkey to keypair (using x-only pubkey from keypair)
        let mut map: BTreeMap<Vec<u8>, secp256k1::KeyPair> = BTreeMap::new();
        // Get x-only public key directly from keypair (this is the correct Kaspa way)
        let schnorr_public_key = keypair.public_key().x_only_public_key().0;
        let script_pub_key_script: Vec<u8> = once(0x20)
            .chain(schnorr_public_key.serialize().into_iter())
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

                // Create message from sighash using from_slice
                let msg = secp256k1::Message::from_slice(sig_hash.as_bytes().as_slice())
                    .map_err(|e| format!("Failed to create message: {}", e))?;

                // Sign with Schnorr using Secp256k1 context
                let sig: [u8; 64] = *secp.sign_schnorr_no_aux_rand(&msg, &schnorr_key).as_ref();

                // Build signature script: OP_DATA_65 + 64-byte signature + 1-byte sighash
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

        // Build JSON transaction for API submission
        let mut json_inputs = Vec::new();
        for input in &signable_tx.tx.inputs {
            json_inputs.push(JsonTransactionInput {
                previous_outpoint: JsonOutPoint {
                    transaction_id: hex::encode(input.previous_outpoint.transaction_id.as_bytes()),
                    index: input.previous_outpoint.index,
                },
                signature_script: hex::encode(&input.signature_script),
                sequence: input.sequence,
                sig_op_count: input.sig_op_count,
            });
        }

        let mut json_outputs = Vec::new();
        for output in &signable_tx.tx.outputs {
            json_outputs.push(JsonTransactionOutput {
                amount: output.value,
                script_public_key: JsonScriptPublicKey {
                    version: output.script_public_key.version(),
                    script: hex::encode(output.script_public_key.script()),
                },
            });
        }

        let json_tx = JsonTransaction {
            version: signable_tx.tx.version as u32,
            inputs: json_inputs,
            outputs: json_outputs,
            lock_time: signable_tx.tx.lock_time,
            subnetwork_id: format!("{}", signable_tx.tx.subnetwork_id),
            gas: 0,
            payload: hex::encode(&signable_tx.tx.payload),
            mass: compute_transaction_mass(&signable_tx.tx),
        };

        Ok(KaspaSignedTransaction {
            tx_hex,
            tx_id: tx_id_hex,
            json_tx,
        })
    }

    pub fn sign_no_payload(
        &mut self,
        private_key: &[u8],
    ) -> Result<KaspaSignedTransaction, String> {
        let secp = Secp256k1::new();
        let keypair = secp256k1::KeyPair::from_seckey_slice(&secp, private_key)
            .map_err(|e| format!("Invalid private key: {}", e))?;

        let (xonly_pubkey, _) = keypair.x_only_public_key();
        let pubkey_bytes: [u8; 32] = xonly_pubkey.serialize();

        eprintln!(
            "DEBUG: X-only public key (transfer): {}",
            hex::encode(&pubkey_bytes)
        );

        let mut signable_tx =
            MutableTransaction::with_entries(self.transaction.clone(), self.utxos.clone());

        let mut map: BTreeMap<Vec<u8>, secp256k1::KeyPair> = BTreeMap::new();
        let schnorr_public_key = keypair.public_key().x_only_public_key().0;
        let script_pub_key_script: Vec<u8> = once(0x20)
            .chain(schnorr_public_key.serialize().into_iter())
            .chain(once(0xac))
            .collect();
        map.insert(script_pub_key_script, keypair);

        let reused_values = SigHashReusedValuesUnsync::new();

        for i in 0..signable_tx.tx.inputs.len() {
            let script = signable_tx.entries[i]
                .as_ref()
                .unwrap()
                .script_public_key
                .script();

            if let Some(schnorr_key) = map.get(script) {
                let sig_hash = calc_schnorr_signature_hash(
                    &signable_tx.as_verifiable(),
                    i,
                    SIG_HASH_ALL,
                    &reused_values,
                );

                eprintln!(
                    "DEBUG: Sighash {} (transfer): {}",
                    i,
                    hex::encode(sig_hash.as_bytes())
                );

                let msg = secp256k1::Message::from_slice(sig_hash.as_bytes().as_slice())
                    .map_err(|e| format!("Failed to create message: {}", e))?;

                let sig: [u8; 64] = *secp.sign_schnorr_no_aux_rand(&msg, &schnorr_key).as_ref();

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
            }
        }

        signable_tx.tx.finalize();
        let mut serialized = Vec::new();
        borsh::BorshSerialize::serialize(&signable_tx.tx, &mut serialized)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let tx_hex = hex::encode(serialized.clone());

        let tx_id = signable_tx.tx.id();
        let tx_id_hex = hex::encode(tx_id.as_bytes());

        eprintln!("DEBUG: Signed tx ID (transfer): {}", tx_id_hex);
        eprintln!("DEBUG: Signed tx hex length (transfer): {}", tx_hex.len());

        let mut json_inputs = Vec::new();
        for input in &signable_tx.tx.inputs {
            json_inputs.push(JsonTransactionInput {
                previous_outpoint: JsonOutPoint {
                    transaction_id: hex::encode(input.previous_outpoint.transaction_id.as_bytes()),
                    index: input.previous_outpoint.index,
                },
                signature_script: hex::encode(&input.signature_script),
                sequence: input.sequence,
                sig_op_count: input.sig_op_count,
            });
        }

        let mut json_outputs = Vec::new();
        for output in &signable_tx.tx.outputs {
            json_outputs.push(JsonTransactionOutput {
                amount: output.value,
                script_public_key: JsonScriptPublicKey {
                    version: output.script_public_key.version(),
                    script: hex::encode(output.script_public_key.script()),
                },
            });
        }

        let json_tx = JsonTransaction {
            version: signable_tx.tx.version as u32,
            inputs: json_inputs,
            outputs: json_outputs,
            lock_time: signable_tx.tx.lock_time,
            subnetwork_id: format!("{}", signable_tx.tx.subnetwork_id),
            gas: 0,
            payload: String::new(),
            mass: compute_transaction_mass(&signable_tx.tx),
        };

        let json_tx_str = serde_json::to_string_pretty(&json_tx)
            .map_err(|e| format!("Failed to serialize JSON tx: {}", e))?;
        eprintln!("DEBUG: JSON transaction:\n{}", json_tx_str);

        Ok(KaspaSignedTransaction {
            tx_hex,
            tx_id: tx_id_hex,
            json_tx,
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
