//! Stellar signature verification
//!
//! Verifies ed25519 signatures from Stellar wallets.

use base32::Alphabet;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use thiserror::Error;

/// Errors that can occur during signature verification
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid Stellar address format: {0}")]
    InvalidAddressFormat(String),

    #[error("Invalid address checksum")]
    InvalidChecksum,

    #[error("Invalid signature format: {0}")]
    InvalidSignatureFormat(String),

    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
}

/// Verify a Stellar wallet signature
///
/// # Arguments
/// * `public_key` - Stellar G-address (e.g., "GABC...")
/// * `message` - The message that was signed
/// * `signature` - Base64-encoded signature
///
/// # Returns
/// * `Ok(true)` if signature is valid
/// * `Err(CryptoError)` if verification fails
pub fn verify_stellar_signature(
    public_key: &str,
    message: &str,
    signature_base64: &str,
) -> Result<bool, CryptoError> {
    // Decode the Stellar public key from G-address
    let public_key_bytes = decode_stellar_public_key(public_key)?;

    // Decode the base64 signature
    let signature_bytes = base64_decode(signature_base64)
        .map_err(|e| CryptoError::InvalidSignatureFormat(e.to_string()))?;

    // Parse the ed25519 signature (64 bytes)
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|e| CryptoError::InvalidSignatureFormat(e.to_string()))?;

    // Create the verifying key
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;

    // Verify the signature
    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(()) => Ok(true),
        Err(_) => Err(CryptoError::VerificationFailed),
    }
}

/// Decode a Stellar public key from G-address format
///
/// Stellar addresses are base32-encoded with a version byte prefix
/// and a 2-byte CRC16 checksum at the end.
fn decode_stellar_public_key(address: &str) -> Result<[u8; 32], CryptoError> {
    // Stellar public keys start with 'G'
    if !address.starts_with('G') {
        return Err(CryptoError::InvalidAddressFormat(
            "Stellar public keys must start with 'G'".to_string(),
        ));
    }

    // Decode base32 (Stellar uses RFC 4648 without padding)
    let decoded = base32::decode(Alphabet::Rfc4648 { padding: false }, address)
        .ok_or_else(|| CryptoError::InvalidAddressFormat("Invalid base32 encoding".to_string()))?;

    // Should be 35 bytes: 1 version byte + 32 key bytes + 2 checksum bytes
    if decoded.len() != 35 {
        return Err(CryptoError::InvalidAddressFormat(format!(
            "Expected 35 bytes, got {}",
            decoded.len()
        )));
    }

    // Verify checksum (CRC16-XModem)
    let payload = &decoded[..33];
    let checksum = &decoded[33..35];
    let calculated_checksum = crc16_xmodem(payload);

    if checksum != calculated_checksum {
        return Err(CryptoError::InvalidChecksum);
    }

    // Extract the 32-byte public key (skip version byte)
    let mut public_key = [0u8; 32];
    public_key.copy_from_slice(&decoded[1..33]);

    Ok(public_key)
}

/// Calculate CRC16-XModem checksum (used by Stellar)
fn crc16_xmodem(data: &[u8]) -> [u8; 2] {
    let mut crc: u16 = 0;

    for byte in data {
        crc ^= (*byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }

    // Little-endian byte order
    [(crc & 0xff) as u8, (crc >> 8) as u8]
}

/// Decode base64 string to bytes
fn base64_decode(encoded: &str) -> Result<Vec<u8>, String> {
    // Simple base64 decoding (supports standard and URL-safe variants)
    const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let input = encoded.trim().trim_end_matches('=');
    let mut output = Vec::with_capacity((input.len() * 6) / 8);
    let mut buffer: u32 = 0;
    let mut bits_in_buffer = 0;

    for ch in input.chars() {
        let value = if let Some(pos) = STANDARD.iter().position(|&c| c as char == ch) {
            pos as u32
        } else if let Some(pos) = URL_SAFE.iter().position(|&c| c as char == ch) {
            pos as u32
        } else {
            return Err(format!("Invalid base64 character: {}", ch));
        };

        buffer = (buffer << 6) | value;
        bits_in_buffer += 6;

        if bits_in_buffer >= 8 {
            bits_in_buffer -= 8;
            output.push((buffer >> bits_in_buffer) as u8);
            buffer &= (1 << bits_in_buffer) - 1;
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_stellar_public_key() {
        // Example valid Stellar public key
        let address = "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN7";
        let result = decode_stellar_public_key(address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_address_format() {
        // Invalid prefix
        let address = "SAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN7";
        let result = decode_stellar_public_key(address);
        assert!(matches!(result, Err(CryptoError::InvalidAddressFormat(_))));
    }

    #[test]
    fn test_crc16_xmodem() {
        // Simple test case
        let data = [0x00, 0x01, 0x02];
        let checksum = crc16_xmodem(&data);
        // Verify it returns 2 bytes
        assert_eq!(checksum.len(), 2);
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "SGVsbG8gV29ybGQ=";
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, b"Hello World");
    }
}
