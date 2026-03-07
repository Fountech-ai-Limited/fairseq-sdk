//! Integration tests for the Fairseq SDK
//! These tests require a running Lighthouse service

use fairseq_sdk::{Config, Prover, Transaction, Verifier};

fn get_test_config() -> Config {
    let mut config = Config::default();
    config.lighthouse.http_url = std::env::var("LIGHTHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    config.lighthouse.api_key = std::env::var("FAIRSEQ_API_KEY").ok();
    config
}

#[test]
fn test_sdk_exports() {
    let _config = Config::default();
    let _tx = Transaction::new("test", 0);
}

#[test]
fn test_config_default() {
    let config = Config::default();
    assert!(!config.lighthouse.http_url.is_empty());
    assert!(config.timeout_secs > 0);
}

#[tokio::test]
async fn test_prover_verifier_creation_returns_result() {
    let mut config = get_test_config();
    config.lighthouse.http_url = "http://127.0.0.1:1".to_string();
    config.lighthouse.ws_url = "ws://127.0.0.1:1".to_string();
    config.lighthouse.timeout_secs = 1;

    let prover = Prover::new(config.clone()).await;
    assert!(prover.is_err(), "Prover should return Err without Lighthouse");

    let verifier = Verifier::new(config).await;
    assert!(verifier.is_err(), "Verifier should return Err without Lighthouse");
}

// Integration tests that require running Lighthouse
// Run with: cargo test --features integration
#[cfg(feature = "integration")]
mod with_lighthouse {
    use super::*;

    #[tokio::test]
    async fn test_full_prove_verify_cycle() {
        let config = get_test_config();
        let prover = Prover::new(config.clone()).await.unwrap();
        let verifier = Verifier::new(config).await.unwrap();

        let now = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let transactions = vec![
            Transaction::new("tx1", now as u64),
            Transaction::new("tx2", now as u64 + 1),
        ];

        let proof = prover.prove(transactions).await;
        assert!(proof.is_ok(), "Proof generation should succeed");

        let proof = proof.unwrap();

        let result = verifier.verify(&proof).await;
        assert!(result.is_ok(), "Proof verification should succeed");

        let verification = result.unwrap();
        assert!(verification.valid, "Proof should be valid");
    }
}
