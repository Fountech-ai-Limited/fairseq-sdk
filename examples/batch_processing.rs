use fairseq_sdk::{Config, Prover, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let prover = Prover::new(config).await?;

    // Simulate a batch
    let mut txs = Vec::new();
    for i in 0..10 {
        txs.push(Transaction::new(format!("0x{:x}", i), 1704067200000000000 + i * 1_000_000));
    }

    let proof = prover.prove(txs).await?;
    println!("Batch proof: {}", proof.id);
    println!("Transactions: {}", proof.transaction_count);

    Ok(())
}


