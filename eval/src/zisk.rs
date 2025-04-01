#[cfg(feature = "zisk")]
use std::{env, process::Command};

#[cfg(feature = "zisk")]
use crate::{
    types::ProgramId,
    utils::{get_elf, time_operation},
};
use crate::{EvalArgs, PerformanceReport};

pub struct ZiskEvaluator;

impl ZiskEvaluator {
    #[cfg(feature = "zisk")]
    pub fn eval(args: &EvalArgs) -> PerformanceReport {
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

        let elf_path = get_elf(args);

        let mut prove = Command::new("cargo-zisk");
        prove.arg("prove").arg("-e").arg(elf_path);

        match args.program {
            ProgramId::Reth => {
                let block_number = args
                    .block_number
                    .expect("Block number is required for Reth program");
                let current_dir =
                    env::current_dir().expect("failed to get current working directory");

                let blocks_dir = current_dir.join("eval").join("blocks");
                let file_path = blocks_dir.join(format!("{}.bin", block_number));
                prove.arg("-i").arg(file_path);
            }
            ProgramId::Fibonacci => {
                let block_number = args
                    .fibonacci_input
                    .expect("Fibonacci input is required for Fibonacci program");
                let current_dir =
                    env::current_dir().expect("failed to get current working directory");

                let blocks_dir = current_dir.join("eval").join("fibonacci");
                let file_path = blocks_dir.join(format!("{}.bin", block_number));
                prove.arg("-i").arg(file_path);
            }
            _ => {}
        };

        let (_status, prove_duration) = time_operation(|| {
            prove
                .arg("-o")
                .arg("proof")
                .arg("-a")
                .status()
                .expect("Failed to execute cargo-zisk command")
        });

        PerformanceReport {
            program,
            prover: args.prover.to_string(),
            shard_size: 0,
            shards: 0,
            cycles: 0,
            speed: 0.0,
            execution_duration: 0.0,
            prove_duration: prove_duration.as_secs_f64(),
            core_prove_duration: prove_duration.as_secs_f64(),
            core_verify_duration: 0.0,
            core_proof_size: 0,
            compress_prove_duration: 0.0,
            compress_verify_duration: 0.0,
            compress_proof_size: 0,
            core_khz: 0.0,
            overall_khz: 0.0,
            wrap_prove_duration: 0.0,
            groth16_prove_duration: 0.0,
            shrink_prove_duration: 0.0,
        }
    }

    #[cfg(not(feature = "zisk"))]
    pub fn eval(_args: &EvalArgs) -> PerformanceReport {
        panic!("Zisk feature is not enabled. Please compile with --features zisk");
    }
}
