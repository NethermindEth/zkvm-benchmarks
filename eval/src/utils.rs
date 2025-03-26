use std::{
    env, fs,
    time::{Duration, Instant},
};

use crate::{
    types::{ProgramId, ProverId},
    EvalArgs,
};

pub fn get_elf(args: &EvalArgs) -> String {
    let mut program_dir = args.program.to_string();
    if args.program == ProgramId::Tendermint || args.program == ProgramId::Reth {
        program_dir += "-";
        program_dir += args.prover.to_string().as_str();
    }

    let current_dir = env::current_dir().expect("Failed to get current working directory");

    let target_name = match args.prover {
        ProverId::SP1 => "riscv32im-succinct-zkvm-elf",
        ProverId::Risc0 => "riscv32im-risc0-zkvm-elf",
        ProverId::Nexus => "riscv32i-unknown-none-elf",
        ProverId::Zisk => "riscv64ima-polygon-ziskos-elf",
        _ => panic!("prover not supported"),
    };

    let elf_path = current_dir.join(format!(
        "benchmarks/{}/target/{}/release/{}",
        program_dir, target_name, program_dir
    ));

    let elf_path_str = elf_path
        .to_str()
        .expect("Failed to convert path to string")
        .to_string();
    println!("elf path: {}", elf_path_str);
    elf_path_str
}

pub fn get_reth_input(args: &EvalArgs) -> Vec<u8> {
    if let Some(block_number) = args.block_number {
        let current_dir = env::current_dir().expect("Failed to get current working directory");
        let blocks_dir = current_dir.join("eval").join("blocks");
        let file_path = blocks_dir.join(format!("{}.bin", block_number));

        match fs::read(&file_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("Failed to read block file: {:?}", e);
                panic!("Unable to read block file: {:?}", e);
            }
        }
    } else {
        panic!("Block number is required for Reth program");
    }
}

pub fn time_operation<T, F: FnOnce() -> T>(operation: F) -> (T, Duration) {
    let start = Instant::now();
    let result = operation();
    let duration = start.elapsed();
    (result, duration)
}
