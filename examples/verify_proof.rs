use fairseq_sdk::{Config, Prover, Transaction, Verifier};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let prover = Prover::new(config.clone()).await?;
    let verifier = Verifier::new(config).await?;

    let transactions = vec![
        Transaction::new("0xabc...", 1704067200000000000),
        Transaction::new("0xdef...", 1704067200100000000),
    ];

    let proof = prover.prove(transactions.clone()).await?;
    let result = verifier.verify_with_transactions(&proof, &transactions).await?;

    if result.valid {
        println!("✓ Proof verified");
    } else {
        println!("✗ Proof invalid: {:?}", result.error);
    }

    Ok(())
}


