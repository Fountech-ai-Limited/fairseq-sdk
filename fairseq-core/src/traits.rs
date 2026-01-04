//! Core traits for Fairseq

use crate::{OrderingRule, Proof, Transaction, VerificationResult};
use async_trait::async_trait;

/// Trait for proof generation
#[async_trait]
pub trait Prover {
    /// The error type for this prover
    type Error: std::error::Error + Send + Sync + 'static;

    /// Generate a temporal ordering proof for the given transactions
    async fn prove(&self, transactions: Vec<Transaction>) -> Result<Proof, Self::Error>;

    /// Generate a proof with custom ordering rule
    async fn prove_with_rule(
        &self,
        transactions: Vec<Transaction>,
        rule: OrderingRule,
    ) -> Result<Proof, Self::Error>;
}

/// Trait for proof verification
#[async_trait]
pub trait Verifier {
    /// The error type for this verifier
    type Error: std::error::Error + Send + Sync + 'static;

    /// Verify a temporal ordering proof
    async fn verify(&self, proof: &Proof) -> Result<VerificationResult, Self::Error>;

    /// Verify a proof against the original transactions
    async fn verify_with_transactions(
        &self,
        proof: &Proof,
        transactions: &[Transaction],
    ) -> Result<VerificationResult, Self::Error>;
}


