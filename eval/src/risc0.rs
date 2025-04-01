#[cfg(feature = "risc0")]
use std::fs;

#[cfg(feature = "risc0")]
use risc0_zkvm::{
    compute_image_id, get_prover_server, ExecutorEnv, ExecutorImpl, ProverOpts, VerifierContext,
};

#[cfg(feature = "risc0")]
use crate::{
    types::ProgramId,
    utils::{get_reth_input, risc0v2::generate_risc0_v2_elf, time_operation},
};

use crate::{EvalArgs, PerformanceReport};

pub struct Risc0Evaluator;

impl Risc0Evaluator {
    #[cfg(feature = "risc0")]
    pub fn eval(args: &EvalArgs) -> PerformanceReport {
        // if args.hashfn != HashFnId::Poseidon {
        //     panic!("Only Poseidon hash function is supported for Risc0.");
        // }

        let program = match args.program {
            ProgramId::Reth => format!(
                "{}_{}",
                args.program.to_string(),
                args.block_number.unwrap().to_string()
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

        // Setup the prover.
        // If the program is Reth or fibonacci, read the block and set it as
        // input. Otherwise, others benchmarks don't have an input.
        let env = match args.program {
            ProgramId::Reth => {
                let input = get_reth_input(args);
                ExecutorEnv::builder()
                    .segment_limit_po2(args.shard_size as u32)
                    .write_slice(&input)
                    .build()
                    .unwrap()
            }
            ProgramId::Fibonacci => ExecutorEnv::builder()
                .segment_limit_po2(args.shard_size as u32)
                .write(&args.fibonacci_input.expect("missing fibonacci input"))
                .expect("Failed to write input to executor")
                .build()
                .unwrap(),
            _ => ExecutorEnv::builder()
                .segment_limit_po2(args.shard_size as u32)
                .build()
                .unwrap(),
        };

        // Compute some statistics.
        let mut exec = ExecutorImpl::from_elf(env, &elf).unwrap();
        //Generate the session.
        let (session, execution_duration) = time_operation(|| exec.run().unwrap());
        let cycles = session.user_cycles;

        let opts = ProverOpts::default();
        let prover = get_prover_server(&opts).unwrap();

        // Generate the proof.
        let ctx = VerifierContext::default();
        let (info, core_prove_duration) =
            time_operation(|| prover.prove_session(&ctx, &session).unwrap());

        let receipt = info.receipt;

        let composite_receipt = receipt.inner.composite().unwrap();
        let num_segments = composite_receipt.segments.len();

        // Get the core proof size by summing across all segments.
        let mut core_proof_size = 0;
        for segment in composite_receipt.segments.iter() {
            core_proof_size += segment.seal.len() * 4;
        }

        // Verify the core proof.
        let ((), core_verify_duration) = time_operation(|| receipt.verify(image_id).unwrap());

        // Now compress the proof with recursion.
        let (compressed_proof, compress_duration) =
            time_operation(|| prover.compress(&ProverOpts::succinct(), &receipt).unwrap());

        // Verify the recursive proof
        let ((), recursive_verify_duration) =
            time_operation(|| compressed_proof.verify(image_id).unwrap());

        let succinct_receipt = compressed_proof.inner.succinct().unwrap();

        // GROTH 16 conversion
        // Bn254 wrapping duration

        let (bn254_proof, wrap_prove_duration) = time_operation(|| {
            prover
                .identity_p254(&compressed_proof.inner.succinct().unwrap())
                .unwrap()
        });
        let seal_bytes = bn254_proof.get_seal_bytes();
        tracing::info!("Running groth16 wrapper");
        let (_, groth16_prove_duration) =
            time_operation(|| risc0_zkvm::stark_to_snark(&seal_bytes).unwrap());

        tracing::info!("Done running groth16");

        // Get the recursive proof size.
        let recursive_proof_size = succinct_receipt.seal.len() * 4;
        let prove_duration = core_prove_duration + compress_duration;

        let core_khz = cycles as f64 / core_prove_duration.as_secs_f64() / 1_000.0;
        let overall_khz = cycles as f64 / prove_duration.as_secs_f64() / 1_000.0;

        // Create the performance report.
        PerformanceReport {
            program,
            prover: args.prover.to_string(),
            //hashfn: args.hashfn.to_string(),
            shard_size: args.shard_size,
            shards: num_segments,
            cycles: cycles as u64,
            speed: (cycles as f64) / prove_duration.as_secs_f64(),
            execution_duration: execution_duration.as_secs_f64(),
            prove_duration: prove_duration.as_secs_f64(),
            core_prove_duration: core_prove_duration.as_secs_f64(),
            core_verify_duration: core_verify_duration.as_secs_f64(),
            core_proof_size,
            compress_prove_duration: compress_duration.as_secs_f64(),
            compress_verify_duration: recursive_verify_duration.as_secs_f64(),
            compress_proof_size: recursive_proof_size,
            core_khz,
            overall_khz,
            wrap_prove_duration: wrap_prove_duration.as_secs_f64(),
            groth16_prove_duration: groth16_prove_duration.as_secs_f64(),
            shrink_prove_duration: 0.0,
        }
    }

    #[cfg(not(feature = "risc0"))]
    pub fn eval(_args: &EvalArgs) -> PerformanceReport {
        panic!("RISC0 feature is not enabled. Please compile with --features risc0");
    }
}
