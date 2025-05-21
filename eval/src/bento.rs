#[cfg(feature = "bento")]
use std::{
    fs,
    time::{Duration, Instant},
};

#[cfg(feature = "bento")]
use bonsai_sdk::blocking::Client;
#[cfg(feature = "bento")]
use risc0_zkvm::{compute_image_id, ExecutorEnv, ExecutorImpl, Receipt};

#[cfg(feature = "bento")]
use crate::{
    types::ProgramId,
    utils::{get_reth_input, risc0v2::generate_risc0_v2_elf, time_operation},
};

use crate::{EvalArgs, PerformanceReport};

#[cfg(feature = "bento")]
const SLEEP_DURATION: Duration = Duration::from_millis(100);

pub struct BentoEvaluator;

impl BentoEvaluator {
    #[cfg(feature = "bento")]
    pub fn eval(args: &EvalArgs) -> PerformanceReport {
        let program = match args.program {
            ProgramId::Reth => format!(
                "{}_{}",
                args.program.to_string(),
                args.block_name.as_deref().unwrap().to_string()
            ),
            ProgramId::Fibonacci => format!(
                "{}_{}",
                args.program.to_string(),
                args.fibonacci_input.unwrap().to_string()
            ),
            _ => args.program.to_string(),
        };

        let elf_path = generate_risc0_v2_elf(args);
        let elf = fs::read(&elf_path).unwrap();
        let image_id = compute_image_id(elf.as_slice()).unwrap();
        let image_id_str = image_id.to_string();

        // Setup the prover.
        // If the program is Reth or fibonacci, read the block and set it as
        // input. Otherwise, others benchmarks don't have an input.
        let (env, input) = match args.program {
            ProgramId::Reth => {
                let input = get_reth_input(args);
                (
                    ExecutorEnv::builder()
                        .segment_limit_po2(args.shard_size as u32)
                        .write_slice(&input)
                        .build()
                        .unwrap(),
                    input,
                )
            }
            ProgramId::Fibonacci => {
                let input = args.fibonacci_input.expect("missing fibonacci input");

                (
                    ExecutorEnv::builder()
                        .segment_limit_po2(args.shard_size as u32)
                        .write(&input)
                        .expect("Failed to write input to executor")
                        .build()
                        .unwrap(),
                    input.to_le_bytes().to_vec(),
                )
            }
            _ => (
                ExecutorEnv::builder()
                    .segment_limit_po2(args.shard_size as u32)
                    .build()
                    .unwrap(),
                vec![],
            ),
        };

        // Compute some statistics.
        let mut exec = ExecutorImpl::from_elf(env, &elf).unwrap();
        //Generate the session.
        let (session, execution_duration) = time_operation(|| exec.run().unwrap());
        let cycles = session.user_cycles;

        // Bento is using the same API of bonsai
        let client = Client::from_parts(
            args.bento_url.clone().expect("missing bento url"),
            String::new(),
            risc0_zkvm::VERSION,
        )
        .unwrap();

        client.upload_img(&image_id_str, elf).unwrap();
        let input_id = client.upload_input(input).unwrap();

        tracing::info!("Asking to bento core and recursion proving");
        let prove_duration;
        let start = Instant::now();
        let session = client
            .create_session(image_id_str, input_id, vec![], false)
            .unwrap();

        let (receipt, _res) = loop {
            let res = session.status(&client).unwrap();
            match res.status.as_ref() {
                "RUNNING" => {
                    std::thread::sleep(SLEEP_DURATION);
                    continue;
                }
                "SUCCEEDED" => {
                    let duration = start.elapsed();
                    prove_duration = duration.as_secs_f64();

                    let receipt_buf = client.receipt_download(&session).unwrap();
                    let receipt: Receipt = bincode::deserialize(&receipt_buf).unwrap();
                    break (receipt, res);
                }
                _ => panic!(
                    "Job failed: {} - {}",
                    session.uuid,
                    res.error_msg.as_ref().unwrap_or(&String::new())
                ),
            }
        };

        let succinct_receipt = receipt.inner.succinct().unwrap();
        let recursive_proof_size = succinct_receipt.seal.len() * 4;

        // Verify the core proof.
        let ((), compress_verify_duration) = time_operation(|| receipt.verify(image_id).unwrap());

        tracing::info!("Asking to bento groth16 proving");
        let groth16_prove_duration;
        let start = Instant::now();
        let snark_session = client.create_snark(session.uuid).unwrap();

        loop {
            let res = snark_session.status(&client).unwrap();
            match res.status.as_ref() {
                "RUNNING" => {
                    std::thread::sleep(SLEEP_DURATION);
                    continue;
                }
                "SUCCEEDED" => {
                    let duration = start.elapsed();
                    groth16_prove_duration = duration.as_secs_f64();

                    let receipt_buf = client.download(&res.output.unwrap()).unwrap();
                    let _receipt: Receipt = bincode::deserialize(&receipt_buf).unwrap();
                    break;
                }
                _ => panic!(
                    "Job failed: {} - {}",
                    snark_session.uuid,
                    res.error_msg.as_ref().unwrap_or(&String::new())
                ),
            }
        }

        let core_khz = 0.0;
        let overall_khz = cycles as f64 / prove_duration / 1_000.0;

        PerformanceReport {
            program,
            prover: args.prover.to_string(),
            shard_size: args.shard_size,
            shards: 0,
            cycles: cycles as u64,
            speed: (cycles as f64) / prove_duration,
            execution_duration: execution_duration.as_secs_f64(),
            prove_duration,
            core_prove_duration: 0.0,
            core_verify_duration: 0.0,
            core_proof_size: 0,
            compress_prove_duration: 0.0,
            compress_verify_duration: compress_verify_duration.as_secs_f64(),
            compress_proof_size: recursive_proof_size,
            core_khz,
            overall_khz,
            wrap_prove_duration: 0.0,
            groth16_prove_duration,
            shrink_prove_duration: 0.0,
        }
    }

    #[cfg(not(feature = "bento"))]
    pub fn eval(_args: &EvalArgs) -> PerformanceReport {
        panic!("Bento feature is not enabled. Please compile with --features bento");
    }
}
