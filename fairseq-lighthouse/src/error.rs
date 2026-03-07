//! Error types for Lighthouse client operations.
//!
//! This module defines the error types that can occur when interacting
//! with the Lighthouse time beacon service.

use thiserror::Error;

/// Errors that can occur during Lighthouse operations.
///
/// # Example
///
/// ```rust
/// use fairseq_lighthouse::LighthouseError;
///
/// fn example() -> Result<(), LighthouseError> {
///     // Lighthouse operations return this error type
///     Ok(())
/// }
/// ```
#[derive(Error, Debug)]
pub enum LighthouseError {
    /// Failed to connect to the Lighthouse service.
    ///
    /// This typically indicates network issues or the service is unavailable.
    #[error("Connection failed: {0}")]
    Connection(String),

    /// The Lighthouse service returned an error response.
    #[error("Lighthouse service error: {message} (code: {code})")]
    ServiceError {
        /// Error code from the service
        code: i32,
        /// Human-readable error message
        message: String,
    },

    /// Request timed out waiting for Lighthouse response.
    #[error("Request timed out after {seconds} seconds")]
    Timeout {
        /// Number of seconds before timeout
        seconds: u64,
    },

    /// Failed to parse response from Lighthouse.
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// The requested epoch was not found.
    #[error("Epoch {epoch_number} not found")]
    EpochNotFound {
        /// The epoch number that was requested
        epoch_number: u64,
    },

    /// Invalid epoch anchor data.
    #[error("Invalid epoch anchor: {0}")]
    InvalidAnchor(String),
}

/// Result type for Lighthouse operations.
pub type Result<T> = std::result::Result<T, LighthouseError>;


