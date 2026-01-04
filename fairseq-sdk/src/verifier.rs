//! Proof verification

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
        let lighthouse = LighthouseClient::new(config.lighthouse.clone());

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


