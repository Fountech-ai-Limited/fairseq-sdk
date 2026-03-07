//! Proof utilities

use fairseq_core::Proof;
use serde::{Deserialize, Serialize};

use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Serialized proof format for transport/storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedProof {
    /// Base64-encoded proof JSON
    pub data: String,
    /// Proof format version
    pub version: u8,
}

impl TryFrom<&Proof> for SerializedProof {
    type Error = fairseq_core::FairseqError;

    fn try_from(proof: &Proof) -> Result<Self, Self::Error> {
        let json = serde_json::to_vec(proof)
            .map_err(|e| fairseq_core::FairseqError::Serialization(e.to_string()))?;
        Ok(Self {
            data: STANDARD.encode(&json),
            version: proof.version,
        })
    }
}

impl SerializedProof {
    /// Serialize the proof wrapper to JSON
    pub fn to_json(&self) -> Result<String, fairseq_core::FairseqError> {
        serde_json::to_string(self)
            .map_err(|e| fairseq_core::FairseqError::Serialization(e.to_string()))
    }
}

impl TryFrom<SerializedProof> for Proof {
    type Error = fairseq_core::FairseqError;

    fn try_from(serialized: SerializedProof) -> Result<Self, Self::Error> {
        let json = STANDARD
            .decode(serialized.data.as_bytes())
            .map_err(|e| fairseq_core::FairseqError::ProofVerification(e.to_string()))?;
        serde_json::from_slice(&json)
            .map_err(|e| fairseq_core::FairseqError::ProofVerification(e.to_string()))
    }
}


