use fairseq_sdk::{Config, Prover, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let prover = Prover::new(config).await?;

    let transactions = vec![
        Transaction::new("0xabc...", 1704067200000000000),
        Transaction::new("0xdef...", 1704067200100000000),
    ];

    let proof = prover.prove(transactions).await?;
    println!("Proof generated: {}", proof.id);
    println!(
        "Epoch range: {} -> {}",
        proof.epoch_start.epoch_number, proof.epoch_end.epoch_number
    );

    Ok(())
}


