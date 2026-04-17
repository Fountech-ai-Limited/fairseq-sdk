//! VDF (Verifiable Delay Function) verification
//!
//! ALPHA PLACEHOLDER -- This module does NOT implement real VDF verification.
//! `verify_vdf_proof()` only checks that the input and output are non-zero,
//! which is enough to catch obviously malformed data but does not perform
//! Wesolowski proof verification. In the current architecture, clients trust
//! the Lighthouse service's Ed25519 signature on epoch data rather than
//! independently verifying the VDF.
//!
//! Planned for v2: integrate a Wesolowski VDF verifier (likely via the
//! `vdf` crate or a custom RSA-group implementation) so that clients can
//! verify epoch timing proofs without trusting the Lighthouse operator.
//!
//! VDF computation itself happens inside the Lighthouse service.
//! This module provides client-side verification utilities.

use crate::hash::sha256;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VdfError {
    #[error("Invalid VDF proof")]
    InvalidProof,
    #[error("VDF verification failed")]
    VerificationFailed,
    #[error("Invalid input")]
    InvalidInput,
}

/// VDF proof data
#[derive(Debug, Clone)]
pub struct VdfProof {
    /// VDF input (typically hash of previous epoch + timestamp)
    pub input: [u8; 32],
    /// VDF output after T iterations
    pub output: [u8; 32],
    /// Proof of correct computation
    pub proof: Vec<u8>,
    /// Number of iterations (T)
    pub iterations: u64,
}

/// Verify a VDF proof.
///
/// ALPHA PLACEHOLDER: This only rejects proofs with all-zero input or output.
/// It does NOT perform Wesolowski verification. Real verification will require
/// an RSA modulus parameter and iterative squaring check. See the Cryptographic
/// Roadmap in the SDK README for the upgrade path.
pub fn verify_vdf_proof(proof: &VdfProof) -> Result<(), VdfError> {
    // TODO(v2): Replace with Wesolowski VDF verification.
    // For now we trust the Lighthouse service's Ed25519 signature on epochs.

    if proof.input == [0u8; 32] {
        return Err(VdfError::InvalidInput);
    }

    if proof.output == [0u8; 32] {
        return Err(VdfError::InvalidProof);
    }

    Ok(())
}

/// Compute VDF input from previous epoch data
pub fn compute_vdf_input(prev_hash: &[u8; 32], timestamp_ns: u64) -> [u8; 32] {
    let mut data = Vec::with_capacity(40);
    data.extend_from_slice(prev_hash);
    data.extend_from_slice(&timestamp_ns.to_le_bytes());
    sha256(&data)
}


