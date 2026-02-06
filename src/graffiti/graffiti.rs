use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraffitiError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Base64 error: {0}")]
    Base64(String),
    #[error("Content too large: {0} bytes (max: {1})")]
    ContentTooLarge(usize, usize),
    #[error("Invalid mimetype: {0}")]
    InvalidMimeType(String),
}

const MAX_PAYLOAD_SIZE: usize = 500;
const MAGIC_BYTES: &[u8] = b"GFX";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraffitiMessage {
    pub version: u8,
    pub timestamp: u64,
    pub content: String,
    pub mimetype: Option<String>,
    pub nonce: u32,
}

impl GraffitiMessage {
    pub fn new(content: String, mimetype: Option<String>) -> Self {
        Self {
            version: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            content,
            mimetype,
            nonce: 0,
        }
    }

    pub fn with_nonce(mut self, nonce: u32) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn validate(&self) -> Result<(), GraffitiError> {
        if self.version != 1 {
            return Err(GraffitiError::InvalidMimeType(
                "Invalid version".to_string(),
            ));
        }

        if let Some(ref mimetype) = self.mimetype {
            if !mimetype.starts_with("text/") && !mimetype.starts_with("image/") {
                return Err(GraffitiError::InvalidMimeType(mimetype.clone()));
            }
        }

        Ok(())
    }
}

pub struct PayloadEncoder;

impl PayloadEncoder {
    pub fn encode(message: &GraffitiMessage) -> Result<Vec<u8>, GraffitiError> {
        message.validate()?;

        let json = serde_json::to_string(message)?;
        let payload_bytes = json.as_bytes();

        if payload_bytes.len() > MAX_PAYLOAD_SIZE {
            return Err(GraffitiError::ContentTooLarge(
                payload_bytes.len(),
                MAX_PAYLOAD_SIZE,
            ));
        }

        let mut result = Vec::with_capacity(MAGIC_BYTES.len() + 1 + payload_bytes.len());
        result.extend_from_slice(MAGIC_BYTES);
        result.push(payload_bytes.len() as u8);
        result.extend_from_slice(payload_bytes);

        Ok(result)
    }

    pub fn decode(data: &[u8]) -> Result<Option<GraffitiMessage>, GraffitiError> {
        if data.len() < MAGIC_BYTES.len() + 1 {
            return Ok(None);
        }

        if &data[..MAGIC_BYTES.len()] != MAGIC_BYTES {
            return Ok(None);
        }

        let payload_len = data[MAGIC_BYTES.len()] as usize;
        let payload_start = MAGIC_BYTES.len() + 1;

        if data.len() < payload_start + payload_len {
            return Ok(None);
        }

        let payload = &data[payload_start..payload_start + payload_len];
        let json_str =
            std::str::from_utf8(payload).map_err(|e| GraffitiError::Base64(e.to_string()))?;

        let message: GraffitiMessage = serde_json::from_str(json_str)?;

        Ok(Some(message))
    }

    pub fn encode_base64(message: &GraffitiMessage) -> Result<String, GraffitiError> {
        let bytes = Self::encode(message)?;
        Ok(BASE64.encode(&bytes))
    }

    pub fn decode_base64(encoded: &str) -> Result<Option<GraffitiMessage>, GraffitiError> {
        let bytes = BASE64
            .decode(encoded)
            .map_err(|e| GraffitiError::Base64(e.to_string()))?;
        Self::decode(&bytes)
    }

    pub fn text_to_graffiti(text: String) -> GraffitiMessage {
        GraffitiMessage::new(text, Some("text/plain".to_string()))
    }

    pub fn image_to_graffiti(base64_data: String) -> GraffitiMessage {
        GraffitiMessage::new(base64_data, Some("image/*".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let message = PayloadEncoder::text_to_graffiti("Hello Kaspa!".to_string());
        assert_eq!(message.version, 1);
        assert_eq!(message.content, "Hello Kaspa!");
        assert_eq!(message.mimetype, Some("text/plain".to_string()));
    }

    #[test]
    fn test_message_encode_decode() {
        let original = PayloadEncoder::text_to_graffiti("Test message".to_string());
        let encoded = PayloadEncoder::encode(&original).unwrap();
        let decoded = PayloadEncoder::decode(&encoded).unwrap().unwrap();
        assert_eq!(decoded.content, original.content);
        assert_eq!(decoded.mimetype, original.mimetype);
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = PayloadEncoder::text_to_graffiti("Base64 test".to_string());
        let encoded = PayloadEncoder::encode_base64(&original).unwrap();
        let decoded = PayloadEncoder::decode_base64(&encoded).unwrap().unwrap();
        assert_eq!(decoded.content, original.content);
    }

    #[test]
    fn test_image_message() {
        let image_data = BASE64.encode(b"fake image data");
        let message = PayloadEncoder::image_to_graffiti(image_data);
        assert_eq!(message.mimetype, Some("image/*".to_string()));
        let encoded = PayloadEncoder::encode(&message).unwrap();
        let decoded = PayloadEncoder::decode(&encoded).unwrap().unwrap();
        assert_eq!(decoded.content, message.content);
    }

    #[test]
    fn test_invalid_data() {
        assert!(PayloadEncoder::decode(b"invalid").unwrap().is_none());
        assert!(PayloadEncoder::decode(&[]).unwrap().is_none());
    }

    #[test]
    fn test_nonce() {
        let message = PayloadEncoder::text_to_graffiti("Test".to_string()).with_nonce(12345);
        let encoded = PayloadEncoder::encode(&message).unwrap();
        let decoded = PayloadEncoder::decode(&encoded).unwrap().unwrap();
        assert_eq!(decoded.nonce, 12345);
    }
}
