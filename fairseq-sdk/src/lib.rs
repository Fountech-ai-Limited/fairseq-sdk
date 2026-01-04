//! Fairseq SDK - Temporal ordering proofs for blockchain protocols
//!
//! Fairseq is the Rust SDK for temporal ordering proofs. Generate cryptographic
//! proof that transactions were sequenced fairly, verifiable by anyone,
//! anchored to trustless time.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use fairseq_sdk::{Config, Prover, Transaction};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a prover with default config
//!     let config = Config::default();
//!     let prover = Prover::new(config).await?;
//!
//!     // Create some transactions
//!     let transactions = vec![
//!         Transaction::new("0xabc...", 1704067200000000000),
//!         Transaction::new("0xdef...", 1704067200100000000),
//!     ];
//!
//!     // Generate a proof
//!     let proof = prover.prove(transactions).await?;
//!     println!("Proof generated: {}", proof.id);
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod proof;
pub mod prover;
pub mod verifier;

// Re-export core types
pub use fairseq_core::{
    EpochAnchor, FairseqError, OrderingRule, Proof, ProofType, Result, Transaction,
    VerificationResult,
};

// Re-export SDK types
pub use config::Config;
pub use prover::Prover;
pub use verifier::Verifier;


