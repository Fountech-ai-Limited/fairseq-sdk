//! SDK configuration

use fairseq_lighthouse::LighthouseConfig;

/// SDK configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Lighthouse configuration
    pub lighthouse: LighthouseConfig,
    /// Enable debug logging
    pub debug: bool,
    /// Maximum transactions per proof
    pub max_transactions: usize,
    /// Proof generation timeout in seconds
    pub timeout_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lighthouse: LighthouseConfig::default(),
            debug: false,
            max_transactions: 10_000,
            timeout_secs: 60,
        }
    }
}

impl Config {
    /// Create a new config with custom Lighthouse URL
    pub fn with_lighthouse_url(
        mut self,
        http_url: impl Into<String>,
        ws_url: impl Into<String>,
    ) -> Self {
        self.lighthouse.http_url = http_url.into();
        self.lighthouse.ws_url = ws_url.into();
        self
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.lighthouse.api_key = Some(api_key.into());
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Load config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(url) = std::env::var("FAIRSEQ_LIGHTHOUSE_URL") {
            config.lighthouse.http_url = url;
        }

        if let Ok(url) = std::env::var("FAIRSEQ_LIGHTHOUSE_WS_URL") {
            config.lighthouse.ws_url = url;
        }

        if let Ok(key) = std::env::var("FAIRSEQ_API_KEY") {
            config.lighthouse.api_key = Some(key);
        }

        if std::env::var("FAIRSEQ_DEBUG").is_ok() {
            config.debug = true;
        }

        config
    }
}


