//! Local proof-construction benchmark
//!
//! Measures the local (CPU-only) compute path that builds a temporal-ordering
//! proof: ordering, hash commitment, and JSON commitment serialization. The
//! Lighthouse epoch lookup is deliberately excluded because it is dominated by
//! network round-trip latency rather than proof construction cost, and the
//! marketing claim ("Targeting sub-second proof generation for batches up to
//! 10K transactions") refers to the local proving step.
//!
//! Methodology:
//!   - Batch sizes: 1, 10, 100, 1_000, 10_000 transactions.
//!   - N = 100 trials per batch size.
//!   - One untimed warm-up iteration per batch size before the timed runs.
//!   - Each trial regenerates the input set with a fresh epoch number to avoid
//!     allocator and branch-predictor caching across trials.
//!   - Wall-clock timings via std::time::Instant.
//!   - Statistics: p50, p95, p99, max (microseconds).
//!
//! Output: CSV on stdout with one header row and one row per batch size.
//!
//! Run with:
//!   cargo run --release -p fairseq-sdk --example bench_local_proof

use std::time::Instant;

use fairseq_core::Transaction;
use fairseq_crypto::hash::hash_ordered_transactions;

const BATCH_SIZES: &[usize] = &[1, 10, 100, 1_000, 10_000];
const TRIALS: usize = 100;
const EPOCH_NUMBER_BASE: u64 = 100_000;

fn build_inputs(n: usize, seed: u64) -> Vec<Transaction> {
    // Deterministic but non-trivial input. Timestamps are shuffled so the sort
    // step does real work; hashes are unique strings of fixed length.
    let mut txs = Vec::with_capacity(n);
    for i in 0..n {
        let mixed = ((i as u64).wrapping_mul(2_654_435_761) ^ seed) % (n as u64).max(1);
        let timestamp_ns = 1_700_000_000_000_000_000u64 + mixed * 1_000;
        let hash = format!("0x{:064x}", (seed << 32) | i as u64);
        txs.push(Transaction::new(hash, timestamp_ns));
    }
    txs
}

fn run_local_proof(transactions: &[Transaction], epoch_number: u64) -> Vec<u8> {
    // Replicates the compute-only portion of Prover::prove_with_rule for
    // OrderingRule::Fifo: sort, verify ordering, hash, commit.

    let mut ordered: Vec<Transaction> = transactions.to_vec();
    ordered.sort_by_key(|t| t.timestamp_ns);

    // Duplicate-detection plus monotonicity check; mirrors verify_ordering.
    let mut seen: std::collections::HashSet<&String> = std::collections::HashSet::new();
    for tx in &ordered {
        assert!(seen.insert(&tx.hash), "duplicate hash");
    }
    for w in ordered.windows(2) {
        assert!(w[0].timestamp_ns <= w[1].timestamp_ns, "ordering invariant violated");
    }

    let tx_data: Vec<(String, u64)> = ordered
        .iter()
        .map(|t| (t.hash.clone(), t.timestamp_ns))
        .collect();
    let transactions_hash = hash_ordered_transactions(&tx_data);

    let commitment = serde_json::json!({
        "version": 1,
        "type": "commitment",
        "epoch": epoch_number,
        "timestamp": 1_700_000_000_000_000_000u64,
        "transaction_count": ordered.len(),
        "first_tx": ordered.first().map(|t| &t.hash),
        "last_tx": ordered.last().map(|t| &t.hash),
        "transactions_hash": transactions_hash,
    });

    // OrderingRule::Fifo is the default and the only rule exercised here;
    // OrderingRule::Priority adds a secondary sort that does not change the
    // order of magnitude of the work.
    serde_json::to_vec(&commitment).expect("commitment serializes")
}

fn percentile(sorted_us: &[u128], p: f64) -> u128 {
    if sorted_us.is_empty() {
        return 0;
    }
    let rank = ((sorted_us.len() as f64 - 1.0) * p).round() as usize;
    sorted_us[rank]
}

fn main() {
    println!(
        "batch_size,trials,p50_us,p95_us,p99_us,max_us,mean_us,p95_ms,p99_ms,sub_second_p95_pass"
    );

    for &batch in BATCH_SIZES {
        // Warm-up.
        let warmup = build_inputs(batch, 0xFEED);
        let _ = run_local_proof(&warmup, EPOCH_NUMBER_BASE);

        let mut timings_us: Vec<u128> = Vec::with_capacity(TRIALS);
        for trial in 0..TRIALS {
            let inputs = build_inputs(batch, 0xABCD ^ trial as u64);
            let epoch = EPOCH_NUMBER_BASE + trial as u64;
            let start = Instant::now();
            let bytes = run_local_proof(&inputs, epoch);
            let elapsed = start.elapsed().as_micros();
            // Sink the bytes so the optimiser does not drop the work.
            std::hint::black_box(bytes);
            timings_us.push(elapsed);
        }

        timings_us.sort_unstable();
        let p50 = percentile(&timings_us, 0.50);
        let p95 = percentile(&timings_us, 0.95);
        let p99 = percentile(&timings_us, 0.99);
        let max = *timings_us.last().unwrap();
        let mean: u128 = timings_us.iter().sum::<u128>() / timings_us.len() as u128;
        let p95_ms = p95 as f64 / 1_000.0;
        let p99_ms = p99 as f64 / 1_000.0;
        let sub_second_pass = if p95_ms < 1000.0 { "pass" } else { "fail" };

        println!(
            "{batch},{TRIALS},{p50},{p95},{p99},{max},{mean},{p95_ms:.3},{p99_ms:.3},{sub_second_pass}"
        );
    }
}
