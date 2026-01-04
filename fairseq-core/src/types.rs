//! Core types for Fairseq

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A transaction to be included in a temporal ordering proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction hash
    pub hash: String,
    /// Arrival timestamp in nanoseconds since Unix epoch
    pub timestamp_ns: u64,
    /// Optional transaction data (for application-specific use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(hash: impl Into<String>, timestamp_ns: u64) -> Self {
        Self {
            hash: hash.into(),
            timestamp_ns,
            data: None,
        }
    }

    /// Create a transaction with additional data
    pub fn with_data(hash: impl Into<String>, timestamp_ns: u64, data: serde_json::Value) -> Self {
        Self {
            hash: hash.into(),
            timestamp_ns,
            data: Some(data),
        }
    }
}

/// A Lighthouse epoch anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochAnchor {
    /// Epoch number
    pub epoch_number: u64,
    /// Epoch timestamp in nanoseconds
    pub timestamp_ns: u64,
    /// VDF output (hex-encoded)
    pub vdf_output: String,
    /// Epoch hash (hex-encoded)
    pub epoch_hash: String,
}

/// A temporal ordering proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// Unique proof identifier
    pub id: String,
    /// Version of the proof format
    pub version: u8,
    /// Type of proof
    pub proof_type: ProofType,
    /// Starting epoch anchor
    pub epoch_start: EpochAnchor,
    /// Ending epoch anchor
    pub epoch_end: EpochAnchor,
    /// Hash of the ordered transaction list
    pub transactions_hash: String,
    /// Number of transactions in the proof
    pub transaction_count: usize,
    /// The cryptographic proof data (format depends on proof_type)
    pub proof_data: Vec<u8>,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Proof generation timestamp
    pub created_at: DateTime<Utc>,
}

/// Type of ordering proof
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    /// First-in-first-out ordering
    TemporalOrdering,
    /// Batch ordering (order within batch doesn't matter)
    BatchOrdering,
    /// Custom ordering rule
    Custom,
}

impl Default for ProofType {
    fn default() -> Self {
        Self::TemporalOrdering
    }
}

/// Ordering rule for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderingRule {
    /// Strict FIFO based on arrival timestamp
    Fifo,
    /// Priority-based ordering (requires priority field in transaction data)
    Priority,
    /// Custom ordering function
    Custom(String),
}

impl Default for OrderingRule {
    fn default() -> Self {
        Self::Fifo
    }
}

/// Result of proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the proof is valid
    pub valid: bool,
    /// Epoch anchors verified
    pub epochs_verified: bool,
    /// Ordering verified
    pub ordering_verified: bool,
    /// Optional error message if invalid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Verification timestamp
    pub verified_at: DateTime<Utc>,
}

impl VerificationResult {
    /// Create a valid verification result
    pub fn valid() -> Self {
        Self {
            valid: true,
            epochs_verified: true,
            ordering_verified: true,
            error: None,
            verified_at: Utc::now(),
        }
    }

    /// Create an invalid verification result with an error message
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            epochs_verified: false,
            ordering_verified: false,
            error: Some(error.into()),
            verified_at: Utc::now(),
        }
    }
}


