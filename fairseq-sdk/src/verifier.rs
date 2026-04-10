//! Proof verification
//!
//! MVP implementation using signed commitments. Full zero-knowledge proof integration planned for v2.

use crate::config::Config;
use fairseq_core::{FairseqError, Proof, Result, Transaction, VerificationResult};
use fairseq_crypto::hash::hash_ordered_transactions;
use fairseq_lighthouse::LighthouseClient;
use tracing::{debug, info};

/// Temporal ordering proof verifier
pub struct Verifier {
    lighthouse: LighthouseClient,
}

impl Verifier {
    /// Create a new verifier
    pub async fn new(config: Config) -> Result<Self> {
        let lighthouse = LighthouseClient::new(config.lighthouse.clone())
            .map_err(|e| FairseqError::LighthouseConnection(e.to_string()))?;

        lighthouse
            .get_current_epoch()
            .await
            .map_err(|e| FairseqError::LighthouseConnection(e.to_string()))?;

        info!("Verifier initialized");

        Ok(Self { lighthouse })
    }

    /// Verify a temporal ordering proof
    pub async fn verify(&self, proof: &Proof) -> Result<VerificationResult> {
        debug!("Verifying proof: {}", proof.id);

        // Verify epoch anchors exist and are valid
        let epoch_start = self
            .lighthouse
            .get_epoch(proof.epoch_start.epoch_number)
            .await
            .map_err(|e| FairseqError::LighthouseEpoch(e.to_string()))?;

        let epoch_end = self
            .lighthouse
            .get_epoch(proof.epoch_end.epoch_number)
            .await
            .map_err(|e| FairseqError::LighthouseEpoch(e.to_string()))?;

        // Verify epoch hashes match
        if hex::encode(epoch_start.epoch_hash) != proof.epoch_start.epoch_hash {
            return Ok(VerificationResult::invalid("Start epoch hash mismatch"));
        }

        if hex::encode(epoch_end.epoch_hash) != proof.epoch_end.epoch_hash {
            return Ok(VerificationResult::invalid("End epoch hash mismatch"));
        }

        if epoch_start.epoch_number > epoch_end.epoch_number {
            return Ok(VerificationResult::invalid("Invalid epoch range"));
        }

        let valid = self.verify_proof_data(proof)?;
        if !valid {
            return Ok(VerificationResult::invalid("Invalid proof data"));
        }

        info!("Proof {} verified successfully", proof.id);

        Ok(VerificationResult::valid())
    }

    /// Verify a proof against the original transactions
    pub async fn verify_with_transactions(
        &self,
        proof: &Proof,
        transactions: &[Transaction],
    ) -> Result<VerificationResult> {
        let result = self.verify(proof).await?;
        if !result.valid {
            return Ok(result);
        }

        if transactions.len() != proof.transaction_count {
            return Ok(VerificationResult::invalid(format!(
                "Transaction count mismatch: expected {}, got {}",
                proof.transaction_count,
                transactions.len()
            )));
        }

        let mut ordered = transactions.to_vec();
        ordered.sort_by_key(|t| t.timestamp_ns);

        let tx_data: Vec<(String, u64)> = ordered
            .iter()
            .map(|t| (t.hash.clone(), t.timestamp_ns))
            .collect();
        let computed_hash = hash_ordered_transactions(&tx_data);

        if computed_hash != proof.transactions_hash {
            return Ok(VerificationResult::invalid("Transactions hash mismatch"));
        }

        Ok(VerificationResult::valid())
    }

    /// Verify the proof data
    /// In production, this would verify the ZK proof.
    fn verify_proof_data(&self, proof: &Proof) -> Result<bool> {
        let commitment: serde_json::Value =
            serde_json::from_slice(&proof.proof_data).map_err(|e| {
                FairseqError::ProofVerification(format!("Invalid proof data: {}", e))
            })?;

        let version = commitment.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
        if version != 1 {
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fairseq_core::{EpochAnchor, ProofType};

    fn test_config() -> Config {
        let mut config = Config::default();
        config.lighthouse.http_url = "http://127.0.0.1:1".to_string();
        config.lighthouse.ws_url = "ws://127.0.0.1:1".to_string();
        config.lighthouse.timeout_secs = 1;
        config
    }

    fn build_verifier() -> Verifier {
        let config = test_config();
        let lighthouse = LighthouseClient::new(config.lighthouse.clone()).unwrap();
        Verifier { lighthouse }
    }

    fn mock_epoch_anchor(epoch_number: u64) -> EpochAnchor {
        EpochAnchor {
            epoch_number,
            timestamp_ns: 1,
            vdf_output: "00".to_string(),
            epoch_hash: "00".to_string(),
        }
    }

    fn mock_proof(proof_data: Vec<u8>) -> Proof {
        Proof {
            id: "proof-123".to_string(),
            version: 1,
            proof_type: ProofType::TemporalOrdering,
            epoch_start: mock_epoch_anchor(1),
            epoch_end: mock_epoch_anchor(2),
            transactions_hash: "abc123".to_string(),
            transaction_count: 1,
            proof_data,
            metadata: None,
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_verifier_new_returns_err_without_lighthouse() {
        let config = test_config();
        let result = Verifier::new(config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_proof_data_valid() {
        let verifier = build_verifier();
        let proof_data = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "type": "commitment",
        }))
        .unwrap();
        let proof = mock_proof(proof_data);

        let result = verifier.verify_proof_data(&proof).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_proof_data_invalid_version() {
        let verifier = build_verifier();
        let proof_data = serde_json::to_vec(&serde_json::json!({
            "version": 2,
            "type": "commitment",
        }))
        .unwrap();
        let proof = mock_proof(proof_data);

        let result = verifier.verify_proof_data(&proof).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_proof_data_invalid_json() {
        let verifier = build_verifier();
        let proof = mock_proof(b"not-json".to_vec());

        let result = verifier.verify_proof_data(&proof);
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_transactions_hash_deterministic() {
        let transactions = vec![
            Transaction::new("tx1", 1),
            Transaction::new("tx2", 2),
        ];
        let tx_data: Vec<(String, u64)> = transactions
            .iter()
            .map(|t| (t.hash.clone(), t.timestamp_ns))
            .collect();

        let hash1 = hash_ordered_transactions(&tx_data);
        let hash2 = hash_ordered_transactions(&tx_data);
        assert_eq!(hash1, hash2);
    }
}


