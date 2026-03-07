//! Proof generation

use crate::config::Config;
use chrono::Utc;
use fairseq_core::{FairseqError, OrderingRule, Proof, ProofType, Result, Transaction};
use fairseq_crypto::hash::hash_ordered_transactions;
use fairseq_lighthouse::LighthouseClient;
use tracing::{debug, info};
use uuid::Uuid;

/// Temporal ordering proof generator
pub struct Prover {
    config: Config,
    lighthouse: LighthouseClient,
}

impl Prover {
    /// Create a new prover
    pub async fn new(config: Config) -> Result<Self> {
        let lighthouse = LighthouseClient::new(config.lighthouse.clone())
            .map_err(|e| FairseqError::LighthouseConnection(e.to_string()))?;

        // Verify Lighthouse connection
        lighthouse
            .get_current_epoch()
            .await
            .map_err(|e| FairseqError::LighthouseConnection(e.to_string()))?;

        info!("Prover initialized, Lighthouse connection verified");

        Ok(Self { config, lighthouse })
    }

    /// Generate a temporal ordering proof for the given transactions
    pub async fn prove(&self, transactions: Vec<Transaction>) -> Result<Proof> {
        self.prove_with_rule(transactions, OrderingRule::Fifo).await
    }

    /// Generate a proof with a custom ordering rule
    pub async fn prove_with_rule(
        &self,
        transactions: Vec<Transaction>,
        rule: OrderingRule,
    ) -> Result<Proof> {
        if transactions.is_empty() {
            return Err(FairseqError::InvalidTransaction(
                "No transactions provided".to_string(),
            ));
        }

        if transactions.len() > self.config.max_transactions {
            return Err(FairseqError::InvalidTransaction(format!(
                "Too many transactions: {} > {}",
                transactions.len(),
                self.config.max_transactions
            )));
        }

        debug!("Generating proof for {} transactions", transactions.len());

        // Get starting epoch
        let epoch_start = self
            .lighthouse
            .get_current_epoch()
            .await
            .map_err(|e| FairseqError::LighthouseEpoch(e.to_string()))?;

        // Sort transactions according to ordering rule
        let ordered = self.apply_ordering(&transactions, &rule)?;

        // Compute transaction commitment
        let tx_data: Vec<(String, u64)> = ordered
            .iter()
            .map(|t| (t.hash.clone(), t.timestamp_ns))
            .collect();
        let transactions_hash = hash_ordered_transactions(&tx_data);

        // Generate proof data
        // In production, this would invoke the T-zkVM to generate a ZK proof.
        // For MVP, we create a signed commitment.
        let proof_data = self.generate_proof_data(&ordered, &epoch_start)?;

        // Get ending epoch
        let epoch_end = self
            .lighthouse
            .get_current_epoch()
            .await
            .map_err(|e| FairseqError::LighthouseEpoch(e.to_string()))?;

        let proof = Proof {
            id: Uuid::new_v4().to_string(),
            version: 1,
            proof_type: ProofType::TemporalOrdering,
            epoch_start: epoch_start.to_anchor(),
            epoch_end: epoch_end.to_anchor(),
            transactions_hash,
            transaction_count: ordered.len(),
            proof_data,
            metadata: None,
            created_at: Utc::now(),
        };

        info!(
            "Proof generated: {} ({} transactions, epochs {}-{})",
            proof.id,
            proof.transaction_count,
            proof.epoch_start.epoch_number,
            proof.epoch_end.epoch_number
        );

        Ok(proof)
    }

    /// Apply ordering rule to transactions
    fn apply_ordering(&self, transactions: &[Transaction], rule: &OrderingRule) -> Result<Vec<Transaction>> {
        let mut ordered = transactions.to_vec();

        match rule {
            OrderingRule::Fifo => {
                ordered.sort_by_key(|t| t.timestamp_ns);
            }
            OrderingRule::Priority => {
                ordered.sort_by(|a, b| {
                    let priority_a = a
                        .data
                        .as_ref()
                        .and_then(|d| d.get("priority"))
                        .and_then(|p| p.as_u64())
                        .unwrap_or(0);
                    let priority_b = b
                        .data
                        .as_ref()
                        .and_then(|d| d.get("priority"))
                        .and_then(|p| p.as_u64())
                        .unwrap_or(0);

                    priority_b.cmp(&priority_a).then(a.timestamp_ns.cmp(&b.timestamp_ns))
                });
            }
            OrderingRule::Custom(_) => {
                return Err(FairseqError::ProofGeneration(
                    "Custom ordering rules not yet implemented".to_string(),
                ));
            }
        }

        self.verify_ordering(&ordered)?;

        Ok(ordered)
    }

    /// Verify that the ordering is valid
    fn verify_ordering(&self, transactions: &[Transaction]) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for tx in transactions {
            if !seen.insert(&tx.hash) {
                return Err(FairseqError::InvalidTransaction(format!(
                    "Duplicate transaction hash: {}",
                    tx.hash
                )));
            }
        }

        for window in transactions.windows(2) {
            if window[0].timestamp_ns > window[1].timestamp_ns {
                return Err(FairseqError::OrderingViolation(format!(
                    "Transaction {} (ts={}) should come after {} (ts={})",
                    window[0].hash,
                    window[0].timestamp_ns,
                    window[1].hash,
                    window[1].timestamp_ns
                )));
            }
        }

        Ok(())
    }

    /// Generate proof data
    /// In production, this would invoke T-zkVM.
    fn generate_proof_data(&self, transactions: &[Transaction], epoch: &fairseq_lighthouse::Epoch) -> Result<Vec<u8>> {
        let commitment = serde_json::json!({
            "version": 1,
            "type": "commitment",
            "epoch": epoch.epoch_number,
            "timestamp": epoch.timestamp_ns,
            "transaction_count": transactions.len(),
            "first_tx": transactions.first().map(|t| &t.hash),
            "last_tx": transactions.last().map(|t| &t.hash),
        });

        serde_json::to_vec(&commitment).map_err(|e| FairseqError::ProofGeneration(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        let mut config = Config::default();
        config.lighthouse.http_url = "http://127.0.0.1:1".to_string();
        config.lighthouse.ws_url = "ws://127.0.0.1:1".to_string();
        config.lighthouse.timeout_secs = 1;
        config
    }

    fn build_prover() -> Prover {
        let config = test_config();
        let lighthouse = LighthouseClient::new(config.lighthouse.clone()).unwrap();
        Prover { config, lighthouse }
    }

    #[tokio::test]
    async fn test_prover_new_returns_err_without_lighthouse() {
        let config = test_config();
        let result = Prover::new(config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_fifo_ordering() {
        let prover = build_prover();
        let transactions = vec![
            Transaction::new("tx2", 2),
            Transaction::new("tx1", 1),
        ];

        let ordered = prover.apply_ordering(&transactions, &OrderingRule::Fifo).unwrap();
        assert_eq!(ordered[0].hash, "tx1");
        assert_eq!(ordered[1].hash, "tx2");
    }

    #[test]
    fn test_duplicate_transaction_hashes_rejected() {
        let prover = build_prover();
        let transactions = vec![
            Transaction::new("tx1", 1),
            Transaction::new("tx1", 2),
        ];

        let result = prover.verify_ordering(&transactions);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_transactions_returns_err() {
        let prover = build_prover();
        let result = prover.prove(Vec::new()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prove_returns_err_without_lighthouse() {
        let prover = build_prover();
        let transactions = vec![Transaction::new("tx1", 1)];

        let result = prover.prove(transactions).await;
        assert!(result.is_err());
    }
}


