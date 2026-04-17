# Fairseq SDK

> **Alpha / MVP** -- This SDK is under active development. The cryptographic
> primitives described below use simplified placeholder implementations suitable
> for integration testing and early adopter feedback. Production-grade
> cryptography (Wesolowski VDF verification, zkVM proof generation) is on the
> roadmap but not yet integrated. See *Cryptographic Roadmap* at the bottom of
> this file for details.

**Temporal ordering proofs for blockchain protocols.**

Fairseq is the Rust SDK for generating and verifying temporal ordering proofs
that transactions were sequenced fairly (FIFO), anchored to verifiable time via
the Lighthouse Network.

## Installation

Add Fairseq to your `Cargo.toml`:

```toml
[dependencies]
fairseq-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use fairseq_sdk::{Config, Prover, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a prover
    let config = Config::default();
    let prover = Prover::new(config).await?;

    // Record transactions with timestamps
    let transactions = vec![
        Transaction::new("0xabc123...", 1704067200000000000), // nanoseconds
        Transaction::new("0xdef456...", 1704067200100000000),
        Transaction::new("0x789ghi...", 1704067200200000000),
    ];

    // Generate a temporal ordering proof
    let proof = prover.prove(transactions).await?;

    println!("Proof ID: {}", proof.id);
    println!("Transactions: {}", proof.transaction_count);
    println!(
        "Epochs: {} -> {}",
        proof.epoch_start.epoch_number, proof.epoch_end.epoch_number
    );

    Ok(())
}
```

## Verification

```rust
use fairseq_sdk::{Config, Verifier};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let verifier = Verifier::new(config).await?;

    // Verify a proof
    let result = verifier.verify(&proof).await?;

    if result.valid {
        println!("✓ Proof is valid");
    } else {
        println!("✗ Proof is invalid: {:?}", result.error);
    }

    Ok(())
}
```

## Configuration

```rust
use fairseq_sdk::Config;

// From environment variables
let config = Config::from_env();

// Or manually
let config = Config::default()
    .with_lighthouse_url(
        "https://lighthouse.fairseq.io",
        "wss://lighthouse.fairseq.io/ws"
    )
    .with_api_key("fsk_...")
    .with_debug(true);
```

## Environment Variables

- `FAIRSEQ_LIGHTHOUSE_URL` - Lighthouse HTTP endpoint
- `FAIRSEQ_LIGHTHOUSE_WS_URL` - Lighthouse WebSocket endpoint
- `FAIRSEQ_API_KEY` - API key for hosted services
- `FAIRSEQ_DEBUG` - Enable debug logging

## Documentation

Full documentation: `https://fairseq.io/docs`

## Cryptographic Roadmap

The current alpha uses simplified implementations for core crypto operations:

| Component | Current (v0.1 alpha) | Planned (v2) |
|---|---|---|
| **VDF verification** | Placeholder that validates non-zero input/output. Lighthouse signature is trusted. | Wesolowski VDF verification with full proof checking. |
| **Proof generation** | Signed commitment (JSON structure bound to epoch data). | zkVM-based zero-knowledge proof generation. |
| **Proof verification** | Commitment structure and epoch anchor validation. | Full ZK proof verification against public parameters. |

These placeholders let you integrate the SDK, test your pipeline end-to-end, and
provide real temporal anchoring via the Lighthouse Network. The proof data is
cryptographically bound to beacon epochs today; what changes in v2 is the
strength of the binding (commitment -> ZK proof) and the independence of
verification (trust Lighthouse signature -> verify VDF locally).

## License

MIT OR Apache-2.0
