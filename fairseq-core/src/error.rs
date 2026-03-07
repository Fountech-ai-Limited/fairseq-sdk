//! Error types for Fairseq

use thiserror::Error;

/// Core Fairseq error type
#[derive(Error, Debug)]
pub enum FairseqError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Lighthouse error
    #[error("Lighthouse error: {0}")]
    Lighthouse(String),

    /// Verification failed error
    #[error("Verification failed: {0}")]
    Verification(String),

    /// Lighthouse connection error
    #[error("Lighthouse connection error: {0}")]
    LighthouseConnection(String),

    /// Lighthouse epoch error
    #[error("Lighthouse epoch error: {0}")]
    LighthouseEpoch(String),

    /// Proof generation error
    #[error("Proof generation error: {0}")]
    ProofGeneration(String),

    /// Proof verification error
    #[error("Proof verification error: {0}")]
    ProofVerification(String),

    /// Invalid transaction error
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Ordering violation
    #[error("Ordering violation: {0}")]
    OrderingViolation(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(String),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Cryptographic error
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for FairseqError {
    fn from(error: serde_json::Error) -> Self {
        FairseqError::Serialization(error.to_string())
    }
}

/// Result type alias for Fairseq operations
pub type Result<T> = std::result::Result<T, FairseqError>;


