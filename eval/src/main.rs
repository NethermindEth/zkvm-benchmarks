mod bento;
mod jolt;
mod nexus;
mod risc0;
mod sp1;
mod types;
mod utils;
mod zisk;

use std::{
    fs::{create_dir_all, OpenOptions},
    path::PathBuf,
};

use bento::BentoEvaluator;
use clap::Parser;
use csv::WriterBuilder;
use eyre::Result;
use jolt::JoltEvaluator;
use nexus::NexusEvaluator;
use serde::Serialize;
use types::{ProgramId, ProverId};

use risc0::Risc0Evaluator;
use sp1::SP1Evaluator;
use zisk::ZiskEvaluator;

#[derive(Parser, Clone)]
#[command(about = "Evaluate the performance of a zkVM on a program.")]
pub struct EvalArgs {
    #[arg(long)]
    program: ProgramId,
    #[arg(long)]
    prover: ProverId,
    // #[arg(long)]
    // hashfn: HashFnId,
    #[arg(long)]
    shard_size: u64,
    #[arg(long)]
    filename: String,
    #[arg(long)]
    block_number: Option<u64>,
    #[arg(long)]
    fibonacci_input: Option<u32>,
    #[arg(long)]
    bento_url: Option<String>,
    #[arg(long)]
    taiko_blocks_dir_suffix: Option<String>,
}

/// The performance report of a zkVM on a program.
#[derive(Debug, Serialize, Default)]
pub struct PerformanceReport {
    /// The program that is being evaluated.
    pub program: String,
    /// The prover that is being evaluated.
    pub prover: String,
    /// The shard size that is being evaluated.
    pub shard_size: u64,
    /// The number of shards.
    pub shards: usize,
    /// The reported number of cycles.
    ///
    /// Note that this number may vary based on the zkVM.
    pub cycles: u64,
    /// The reported speed in cycles per second.
    pub speed: f64,
    /// The reported duration of the execution in seconds.
    pub execution_duration: f64,
    /// The reported duration of the prover in seconds.
    pub prove_duration: f64,
    /// The reported duration of the core proving time in seconds.
    pub core_prove_duration: f64,
    /// The reported duration of the verifier in seconds.
    pub core_verify_duration: f64,
    /// The size of the core proof.
    pub core_proof_size: usize,
    /// The reported duration of the recursive proving time in seconds.
    pub compress_prove_duration: f64,
    /// The reported duration of the verifier in seconds.
    pub compress_verify_duration: f64,
    /// The size of the recursive proof in bytes.
    pub compress_proof_size: usize,
    /// The speed of the core proving time in KHz.
    pub core_khz: f64,
    /// The overall speed in KHz.
    pub overall_khz: f64,
    /// The reported duration of the shrink proving time in seconds.
    pub shrink_prove_duration: f64,
    /// The reported duration of the wrap proving time in seconds.
    pub wrap_prove_duration: f64,
    /// The reported duration of the groth16 proving time in seconds.
    pub groth16_prove_duration: f64,
    /// The size of the groth16 proof in bytes.
    pub groth16_proof_size: usize,
    /// The reported duration of the PLONK proving time in seconds.
    pub plonk_prove_duration: f64,
    /// The size of the PLONK proof in bytes.
    pub plonk_proof_size: usize,
}

fn main() -> Result<()> {
    let args = EvalArgs::parse();

    if args.prover != ProverId::SP1 {
        // Initialize tracing
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .compact()
            .init();
    }

    // Select the correct implementation based on the prover.
    let report = match args.prover {
        ProverId::Risc0 => Risc0Evaluator::eval(&args),
        ProverId::SP1 => SP1Evaluator::eval(&args),
        ProverId::Jolt => JoltEvaluator::eval(&args),
        ProverId::Nexus => NexusEvaluator::eval(&args),
        ProverId::Zisk => ZiskEvaluator::eval(&args),
        ProverId::Bento => BentoEvaluator::eval(&args),
    };

    // Create the results directory if it doesn't exist.
    let results_dir = PathBuf::from("results");
    create_dir_all(&results_dir)?;

    // Create the file
    let filename = format!("{}_{}.csv", args.filename, env!("VERGEN_GIT_SHA"));
    let path = results_dir.join(filename);

    // Check if file exists and get its size
    let file_exists = path.exists();

    // Create a CSV writer with appropriate configuration
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(file_exists) // Only append if file exists
        .open(path.clone())?;

    let mut writer = WriterBuilder::new()
        .has_headers(!file_exists) // Write headers only for new files
        .from_writer(file);

    // Serialize the report - headers will be written automatically for new files
    writer.serialize(&report)?;
    writer.flush()?;

    let latest_filename = format!("{}_latest.csv", args.filename);
    let latest_path = results_dir.join(latest_filename);
    std::fs::copy(&path, &latest_path)?;

    Ok(())
}
