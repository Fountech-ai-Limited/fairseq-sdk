//! Lighthouse client errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LighthouseError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Epoch not found: {0}")]
    EpochNotFound(u64),

    #[error("Invalid epoch: {0}")]
    InvalidEpoch(String),

    #[error("Subscription error: {0}")]
    Subscription(String),
}


