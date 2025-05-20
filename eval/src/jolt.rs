#[cfg(feature = "jolt")]
use jolt_sdk::{
    host::Program, Jolt, JoltHyperKZGProof, JoltPreprocessing, RV32IJoltVM, Serializable,
};

#[cfg(feature = "jolt")]
use fibonacci::{
    analyze_func as analyze_fibonacci, preprocess_func as preprocess_fibonacci,
    prove_func as prove_fibonacci,
};
#[cfg(feature = "jolt")]
use loop_j::{
    analyze_func as analyze_loop, preprocess_func as preprocess_loop, prove_func as prove_loop,
};
#[cfg(feature = "jolt")]
use tendermint_j::{
    analyze_func as analyze_tendermint, preprocess_func as preprocess_tendermint,
    prove_func as prove_tendermint,
};

#[cfg(feature = "jolt")]
use crate::{utils::time_operation, ProgramId};

use crate::{EvalArgs, PerformanceReport};

pub struct JoltEvaluator;

impl JoltEvaluator {
    #[cfg(feature = "jolt")]
    pub fn eval(args: &EvalArgs) -> PerformanceReport {
        let (analyze, preprocess, prove): (
            Box<dyn Fn() -> _>,
            fn() -> (Program, JoltPreprocessing<4, _, _, _>),
            Box<dyn Fn(Program, JoltPreprocessing<4, _, _, _>) -> ((), JoltHyperKZGProof)>,
        ) = match args.program {
            ProgramId::Fibonacci => {
                let analyze = Box::new(|| {
                    analyze_fibonacci(args.fibonacci_input.expect("missing fibonacci input"))
                });
                let prove = Box::new(|program, preprocessing| {
                    prove_fibonacci(
                        program,
                        preprocessing,
                        args.fibonacci_input.expect("missing fibonacci input"),
                    )
                });
                (analyze, preprocess_fibonacci, prove)
            }
            ProgramId::Loop => {
                let analyze = Box::new(|| analyze_loop());
                let prove = Box::new(|program, preprocessing| prove_loop(program, preprocessing));
                (analyze, preprocess_loop, prove)
            }
            ProgramId::Tendermint => {
                let analyze = Box::new(|| analyze_tendermint());
                let prove =
                    Box::new(|program, preprocessing| prove_tendermint(program, preprocessing));
                (analyze, preprocess_tendermint, prove)
            }
            _ => panic!("not implemented yet"),
        };

        // Get the total cycles of the program
        let summary = analyze();
        let instruction_count = summary.analyze::<jolt_sdk::F>();
        let total_cycles = instruction_count
            .iter()
            .map(|(_, count)| count)
            .sum::<usize>();

        // Generate the program and arithmetization
        let ((program, preprocessing), execution_duration) = time_operation(|| preprocess());

        // Generate the proof
        let ((_, proof), prove_duration) =
            time_operation(|| prove(program.clone(), preprocessing.clone()));

        // Get the proof size
        let proof_size = proof.size().expect("failed to get proof size");

        // Verify the proof
        let (_, verify_duration) = time_operation(|| {
            RV32IJoltVM::verify(preprocessing, proof.proof, proof.commitments, None)
        });

        let core_khz = total_cycles as f64 / prove_duration.as_secs_f64() / 1_000.0;
        let overall_khz = total_cycles as f64 / prove_duration.as_secs_f64() / 1_000.0;

        PerformanceReport {
            program: args.program.to_string(),
            prover: args.prover.to_string(),
            shard_size: 0,
            shards: 0,
            cycles: total_cycles as u64,
            speed: (total_cycles as f64) / prove_duration.as_secs_f64(),
            execution_duration: execution_duration.as_secs_f64(),
            prove_duration: prove_duration.as_secs_f64(),
            core_prove_duration: prove_duration.as_secs_f64(),
            core_verify_duration: verify_duration.as_secs_f64(),
            core_proof_size: proof_size,
            compress_prove_duration: 0.0,
            compress_verify_duration: 0.0,
            compress_proof_size: 0,
            core_khz,
            overall_khz,
            shrink_prove_duration: 0.0,
            wrap_prove_duration: 0.0,
            groth16_prove_duration: 0.0,
            groth16_proof_size: 0,
            plonk_prove_duration: 0.0,
            plonk_proof_size: 0,
        }
    }

    #[cfg(not(feature = "jolt"))]
    pub fn eval(_args: &EvalArgs) -> PerformanceReport {
        panic!("Jolt feature is not enabled. Please compile with --features jolt")
    }
}
