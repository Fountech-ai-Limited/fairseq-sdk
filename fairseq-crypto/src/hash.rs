//! Hashing utilities

use sha2::{Digest, Sha256};

/// Compute SHA-256 hash of input bytes
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute SHA-256 hash and return as hex string
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256(data))
}

/// Compute hash of a transaction list for proof commitment
pub fn hash_transactions(hashes: &[String]) -> String {
    let mut hasher = Sha256::new();
    for hash in hashes {
        hasher.update(hash.as_bytes());
        hasher.update(b"|"); // Separator
    }
    hex::encode(hasher.finalize())
}

/// Compute hash of ordered transaction data
pub fn hash_ordered_transactions(transactions: &[(String, u64)]) -> String {
    let mut hasher = Sha256::new();
    for (hash, timestamp) in transactions {
        hasher.update(hash.as_bytes());
        hasher.update(b":");
        hasher.update(timestamp.to_le_bytes());
        hasher.update(b"|");
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex(b"hello");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}


