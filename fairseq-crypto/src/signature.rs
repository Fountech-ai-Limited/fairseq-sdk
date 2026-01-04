//! Ed25519 signature utilities

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Signing failed: {0}")]
    SigningFailed(String),
}

/// Generate a new Ed25519 keypair
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Sign a message with Ed25519
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// Verify an Ed25519 signature
pub fn verify(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> Result<(), SignatureError> {
    verifying_key
        .verify(message, signature)
        .map_err(|_| SignatureError::InvalidSignature)
}

/// Parse a verifying key from bytes
pub fn parse_verifying_key(bytes: &[u8]) -> Result<VerifyingKey, SignatureError> {
    VerifyingKey::try_from(bytes).map_err(|_| SignatureError::InvalidPublicKey)
}

/// Parse a signature from bytes
pub fn parse_signature(bytes: &[u8]) -> Result<Signature, SignatureError> {
    Signature::try_from(bytes).map_err(|_| SignatureError::InvalidSignature)
}


