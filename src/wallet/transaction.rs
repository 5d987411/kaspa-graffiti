use secp256k1::{Message, Secp256k1};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::key::KeyPair;

// Blake2b hash function for Kaspa transaction signing
fn blake2b_hash(data: &[u8]) -> [u8; 32] {
    use blake2::{Blake2b, Digest};
    let mut hasher = Blake2b::<blake2::digest::consts::U32>::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Input index out of bounds")]
    InvalidInputIndex,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Signing error: {0}")]
    SigningError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub txid: String,
    pub vout: u32,
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
    pub script_pubkey_version: u16, // For BIP-143 sighash
    pub sequence: u32,
    pub sig_op_count: u8,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub address: String,
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ScriptData {
    pub data: Vec<u8>,
    pub is_graffiti: bool,
}

impl ScriptData {
    pub fn new_graffiti(data: Vec<u8>) -> Self {
        Self {
            data,
            is_graffiti: true,
        }
    }

    pub fn new_op_return(data: Vec<u8>) -> Self {
        Self {
            data,
            is_graffiti: true,
        }
    }

    pub fn to_script_vec(&self) -> Vec<u8> {
        let mut script = Vec::new();
        // OP_RETURN for public unencrypted messages
        script.push(0x6a); // OP_RETURN
        script.push(self.data.len() as u8);
        script.extend_from_slice(&self.data);
        script
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
    pub payload: Option<ScriptData>,
    pub fee: u64,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            version: 0,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
            payload: None,
            fee: 0,
        }
    }

    pub fn with_payload(payload: ScriptData) -> Self {
        let mut tx = Self::new();
        tx.payload = Some(payload);
        tx
    }

    pub fn add_input(&mut self, txid: String, vout: u32, amount: u64, script_pubkey: Vec<u8>) {
        self.inputs.push(TxInput {
            txid,
            vout,
            amount,
            script_pubkey,
            script_pubkey_version: 0, // Standard version
            sequence: 0,
            sig_op_count: 1,
            signature: None,
            public_key: None,
        });
    }

    pub fn add_output(&mut self, address: String, amount: u64, script_pubkey: Vec<u8>) {
        self.outputs.push(TxOutput {
            address,
            amount,
            script_pubkey,
        });
    }

    pub fn estimate_fee(&self, fee_rate: u64) -> u64 {
        let base_size = 12;
        let input_size = 36 + 73 + 33 + 4;
        let output_size = 8 + 1 + 34;

        let payload_size = if let Some(payload) = &self.payload {
            2 + payload.data.len()
        } else {
            0
        };

        let total_size = base_size
            + (self.inputs.len() * input_size)
            + (self.outputs.len() * output_size)
            + payload_size;

        ((total_size as u64 + 999) / 1000) * fee_rate
    }

    /// Compute BIP-143 style sighash for a specific input
    /// Uses Blake2b hashing (Kaspa standard)
    pub fn get_signature_message(&self, input_index: usize) -> Result<Vec<u8>, TransactionError> {
        if input_index >= self.inputs.len() {
            return Err(TransactionError::InvalidInputIndex);
        }

        // Kaspa BIP-143 style sighash
        let sighash_type: u8 = 0x01;

        // Compute all the hashes
        let previous_outputs_hash = self.hash_previous_outputs();
        let sequences_hash = self.hash_sequences();
        let sig_op_counts_hash = self.hash_sig_op_counts();
        let outputs_hash = self.hash_outputs();
        let payload_hash = self.hash_payload();

        let input = &self.inputs[input_index];

        eprintln!(
            "DEBUG: Computing Kaspa BIP-143 sighash for input {}:",
            input_index
        );
        eprintln!("  txid: {}", input.txid);
        eprintln!("  vout: {}", input.vout);
        eprintln!("  amount: {}", input.amount);
        eprintln!("  script_pubkey: {}", hex::encode(&input.script_pubkey));

        let mut buffer = Vec::new();

        // 1. tx.Version (2-bytes LE)
        buffer.extend_from_slice(&(self.version as u16).to_le_bytes());

        // 2. previousOutputsHash (32 bytes)
        buffer.extend_from_slice(&previous_outputs_hash);

        // 3. sequencesHash (32 bytes)
        buffer.extend_from_slice(&sequences_hash);

        // 4. sigOpCountsHash (32 bytes)
        buffer.extend_from_slice(&sig_op_counts_hash);

        // 5. txIn.PreviousOutpoint.TransactionID (32 bytes - big endian from API, use as-is)
        let txid_bytes = hex::decode(&input.txid)
            .map_err(|e| TransactionError::SerializationError(e.to_string()))?;
        buffer.extend_from_slice(&txid_bytes);

        // 6. txIn.PreviousOutpoint.Index (4-bytes LE)
        buffer.extend_from_slice(&input.vout.to_le_bytes());

        // 7. txIn.PreviousOutput.ScriptPubKeyVersion (2-bytes LE)
        buffer.extend_from_slice(&input.script_pubkey_version.to_le_bytes());

        // 8. txIn.PreviousOutput.ScriptPublicKey.length (8-bytes LE)
        buffer.extend_from_slice(&(input.script_pubkey.len() as u64).to_le_bytes());

        // 9. txIn.PreviousOutput.ScriptPublicKey
        buffer.extend_from_slice(&input.script_pubkey);

        // 10. txIn.PreviousOutput.Value (8-bytes LE)
        buffer.extend_from_slice(&input.amount.to_le_bytes());

        // 11. txIn.Sequence (8-bytes LE)
        buffer.extend_from_slice(&(input.sequence as u64).to_le_bytes());

        // 12. txIn.SigOpCount (1-byte)
        buffer.push(input.sig_op_count);

        // 13. outputsHash (32 bytes)
        buffer.extend_from_slice(&outputs_hash);

        // 14. tx.Locktime (8-bytes LE)
        buffer.extend_from_slice(&self.lock_time.to_le_bytes());

        // 15. tx.SubnetworkID (20 bytes)
        let subnetwork_id = if self.payload.is_some() {
            hex::decode("0100000000000000000000000000000000000000")
        } else {
            hex::decode("0000000000000000000000000000000000000000")
        }
        .map_err(|e| TransactionError::SerializationError(e.to_string()))?;
        buffer.extend_from_slice(&subnetwork_id);

        // 16. tx.Gas (8-bytes LE)
        buffer.extend_from_slice(&0u64.to_le_bytes());

        // 17. payloadHash (32 bytes)
        buffer.extend_from_slice(&payload_hash);

        // 18. SigHash type (1-byte)
        buffer.push(sighash_type);

        eprintln!("DEBUG: BIP-143 buffer length: {}", buffer.len());
        eprintln!(
            "DEBUG: BIP-143 buffer (first 64 bytes): {}",
            hex::encode(&buffer[..64.min(buffer.len())])
        );

        // Return Blake2b hash
        let hash = blake2b_hash(&buffer);
        eprintln!("DEBUG: Final BIP-143 sighash: {}", hex::encode(&hash));
        Ok(hash.to_vec())
    }

    /// Hash of all input outpoints (txid + index)
    fn hash_previous_outputs(&self) -> [u8; 32] {
        let mut buffer = Vec::new();
        for input in &self.inputs {
            if let Ok(txid_bytes) = hex::decode(&input.txid) {
                buffer.extend_from_slice(&txid_bytes);
                buffer.extend_from_slice(&input.vout.to_le_bytes());
            }
        }
        blake2b_hash(&buffer)
    }

    /// Hash of all input sequences
    fn hash_sequences(&self) -> [u8; 32] {
        let mut buffer = Vec::new();
        for input in &self.inputs {
            // Kaspa uses 8-byte sequence in sighash
            buffer.extend_from_slice(&(input.sequence as u64).to_le_bytes());
        }
        blake2b_hash(&buffer)
    }

    /// Hash of all input sigOpCounts
    fn hash_sig_op_counts(&self) -> [u8; 32] {
        let mut buffer = Vec::new();
        for input in &self.inputs {
            buffer.push(input.sig_op_count);
        }
        blake2b_hash(&buffer)
    }

    /// Hash of all outputs
    fn hash_outputs(&self) -> [u8; 32] {
        let mut buffer = Vec::new();
        for output in &self.outputs {
            // Format: Value (8 bytes) + ScriptPublicKey.Version (2 bytes) + ScriptPublicKey.Script
            // NOT including the script length!
            buffer.extend_from_slice(&output.amount.to_le_bytes());
            buffer.extend_from_slice(&0u16.to_le_bytes()); // ScriptPublicKey.Version
            buffer.extend_from_slice(&output.script_pubkey);
        }
        blake2b_hash(&buffer)
    }

    /// Hash of transaction payload
    fn hash_payload(&self) -> [u8; 32] {
        if let Some(payload) = &self.payload {
            blake2b_hash(&payload.data)
        } else {
            [0u8; 32]
        }
    }

    pub fn serialize_for_signing(&self) -> Result<Vec<u8>, TransactionError> {
        // For compatibility - just return empty, we use get_signature_message now
        Ok(Vec::new())
    }

    pub fn serialize(&self) -> Result<Vec<u8>, TransactionError> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&self.version.to_le_bytes());
        buffer.push(self.inputs.len() as u8);

        for input in &self.inputs {
            let txid_bytes = hex::decode(&input.txid)
                .map_err(|e| TransactionError::SerializationError(e.to_string()))?;
            buffer.extend_from_slice(&txid_bytes);
            buffer.extend_from_slice(&input.vout.to_le_bytes());

            buffer.push(input.script_pubkey.len() as u8);
            buffer.extend_from_slice(&input.script_pubkey);
        }

        buffer.push(self.outputs.len() as u8);
        for output in &self.outputs {
            let address_bytes = output.address.as_bytes();
            buffer.push(address_bytes.len() as u8);
            buffer.extend_from_slice(address_bytes);
            buffer.extend_from_slice(&output.amount.to_le_bytes());
        }

        buffer.extend_from_slice(&self.lock_time.to_le_bytes());

        if let Some(payload) = &self.payload {
            buffer.push(0xba_u8);
            buffer.extend_from_slice(&payload.to_script_vec());
        }

        Ok(buffer)
    }

    /// Serialize transaction to JSON format for REST API submission
    pub fn to_json(&self) -> Result<serde_json::Value, TransactionError> {
        use serde_json::json;

        let mut inputs = Vec::new();
        for input in &self.inputs {
            // Build signature script for P2PK
            // Kaspa uses BIP-340 Schnorr signatures with hash type
            // Format: <65-byte sig: 64-byte Schnorr + 1-byte hashtype (0x01)>
            let sig_script = if let Some(sig) = &input.signature {
                // Kaspa expects 65-byte signature (64-byte Schnorr + 0x01 hashtype)
                if sig.len() == 65 {
                    // Use OP_PUSHBYTES_65 + signature + hashtype
                    let mut script = vec![0x41u8]; // OP_PUSHBYTES_65
                    script.extend_from_slice(sig);
                    eprintln!(
                        "DEBUG: P2PK signature script (65-byte sig, {} bytes): {}",
                        script.len(),
                        hex::encode(&script)
                    );
                    hex::encode(script)
                } else {
                    // Fallback for unexpected lengths
                    let mut script = vec![sig.len() as u8];
                    script.extend_from_slice(sig);
                    hex::encode(script)
                }
            } else {
                hex::encode(&input.script_pubkey)
            };

            inputs.push(json!({
                "previousOutpoint": {
                    "transactionId": input.txid,
                    "index": input.vout
                },
                "signatureScript": sig_script,
                "sequence": 0,
                "sigOpCount": 1
            }));
        }

        let mut outputs = Vec::new();
        for output in &self.outputs {
            outputs.push(json!({
                "amount": output.amount,
                "scriptPublicKey": {
                    "version": 0,
                    "scriptPublicKey": hex::encode(&output.script_pubkey)
                }
            }));
        }

        // Build base transaction
        let mut tx_json = json!({
            "version": self.version,
            "inputs": inputs,
            "outputs": outputs,
            "lockTime": self.lock_time,
            "subnetworkId": "0000000000000000000000000000000000000000"
        });

        // Add payload if present (Kaspa uses transaction payload field, not OP_RETURN)
        if let Some(payload) = &self.payload {
            tx_json["payload"] = json!(hex::encode(&payload.data));
        }

        Ok(tx_json)
    }

    pub fn sign_input(
        &mut self,
        input_index: usize,
        keypair: &KeyPair,
    ) -> Result<(), TransactionError> {
        if input_index >= self.inputs.len() {
            return Err(TransactionError::InvalidInputIndex);
        }

        let message_data = self
            .get_signature_message(input_index)
            .map_err(|e| TransactionError::SigningError(e.to_string()))?;

        eprintln!(
            "DEBUG: Signing input {} with sighash: {}",
            input_index,
            hex::encode(&message_data)
        );

        // message_data is already the Blake2b hash of the sighash serialization
        let message = Message::from_slice(&message_data)
            .map_err(|e| TransactionError::SigningError(e.to_string()))?;

        let secp = Secp256k1::new();

        // Kaspa uses Schnorr signatures (BIP-340), not ECDSA
        // Convert to x-only public key for Schnorr
        let xonly_keypair = secp256k1::KeyPair::from_seckey_slice(&secp, &keypair.to_bytes())
            .map_err(|e| TransactionError::SigningError(e.to_string()))?;

        let signature = secp.sign_schnorr_no_aux_rand(&message, &xonly_keypair);

        // Kaspa uses BIP-340 Schnorr signatures
        // Append SIGHASH_ALL (0x01) to signature for Kaspa
        let sighash_type: u8 = 0x01;
        let mut sig_bytes = signature.as_ref().to_vec();
        sig_bytes.push(sighash_type);

        eprintln!(
            "DEBUG: Schnorr signature with hashtype ({} bytes): {}",
            sig_bytes.len(),
            hex::encode(&sig_bytes)
        );

        // Get x-only public key (32 bytes) for Kaspa
        let (xonly_pubkey, _) = xonly_keypair.x_only_public_key();
        let xonly_pubkey_bytes: [u8; 32] = xonly_pubkey.serialize();
        eprintln!(
            "DEBUG: X-only public key (32 bytes): {}",
            hex::encode(&xonly_pubkey_bytes)
        );

        self.inputs[input_index].signature = Some(sig_bytes);
        self.inputs[input_index].public_key = Some(xonly_pubkey_bytes.to_vec());

        Ok(())
    }

    fn sha256(data: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    pub fn compute_fee(&mut self, suggested_fee: u64) {
        self.fee = suggested_fee;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new();
        assert_eq!(tx.version, 1);
        assert!(tx.inputs.is_empty());
        assert!(tx.outputs.is_empty());
    }

    #[test]
    fn test_add_input_output() {
        let mut tx = Transaction::new();
        tx.add_input(
            "abc123def456789".to_string(),
            0,
            1000000,
            vec![
                0x76, 0xa9, 0x14, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
                0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x88, 0xac,
            ],
        );
        tx.add_output("kaspa:abc123".to_string(), 900000, vec![]);

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
    }

    #[test]
    fn test_script_data() {
        let payload = b"hello world";
        let script = ScriptData::new_graffiti(payload.to_vec());
        assert!(script.is_graffiti);
        assert_eq!(script.data, payload);
    }

    #[test]
    fn test_fee_estimation() {
        let mut tx = Transaction::new();
        tx.add_input(
            "abc123".to_string(),
            0,
            1000000,
            vec![
                0x76, 0xa9, 0x14, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
                0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x88, 0xac,
            ],
        );
        tx.add_output("kaspa:xyz".to_string(), 900000, vec![]);

        let fee = tx.estimate_fee(1000);
        assert!(fee > 0);
    }
}
