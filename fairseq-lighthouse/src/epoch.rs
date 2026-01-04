//! Lighthouse epoch types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Lighthouse epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    /// Monotonically increasing epoch number
    pub epoch_number: u64,
    /// Timestamp in nanoseconds since Unix epoch
    pub timestamp_ns: u64,
    /// VDF input (hash of prev_epoch || timestamp)
    #[serde(with = "hex_bytes")]
    pub vdf_input: [u8; 32],
    /// VDF computation result
    #[serde(with = "hex_bytes")]
    pub vdf_output: [u8; 32],
    /// Proof of correct VDF computation
    #[serde(with = "hex_bytes_vec")]
    pub vdf_proof: Vec<u8>,
    /// Hash of previous epoch
    #[serde(with = "hex_bytes")]
    pub prev_hash: [u8; 32],
    /// Hash of this epoch
    #[serde(with = "hex_bytes")]
    pub epoch_hash: [u8; 32],
    /// Ed25519 signature by Lighthouse operator
    #[serde(with = "hex_bytes_64")]
    pub signature: [u8; 64],
}

impl Epoch {
    /// Get the epoch timestamp as a DateTime
    pub fn timestamp(&self) -> DateTime<Utc> {
        let secs = (self.timestamp_ns / 1_000_000_000) as i64;
        let nsecs = (self.timestamp_ns % 1_000_000_000) as u32;
        DateTime::from_timestamp(secs, nsecs).unwrap_or_default()
    }

    /// Convert to EpochAnchor for proof generation
    pub fn to_anchor(&self) -> fairseq_core::EpochAnchor {
        fairseq_core::EpochAnchor {
            epoch_number: self.epoch_number,
            timestamp_ns: self.timestamp_ns,
            vdf_output: hex::encode(self.vdf_output),
            epoch_hash: hex::encode(self.epoch_hash),
        }
    }
}

/// Hex serialization helpers
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        bytes.try_into()
            .map_err(|_| serde::de::Error::custom("invalid length"))
    }
}

mod hex_bytes_64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        bytes.try_into()
            .map_err(|_| serde::de::Error::custom("invalid length"))
    }
}

mod hex_bytes_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}


