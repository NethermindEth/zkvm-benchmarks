use std::fmt::Display;
use clap::ValueEnum;

/// An identifier used to select the prover to evaluate.
#[derive(ValueEnum, Clone, PartialEq)]
pub enum ProverId {
    Risc0,
    SP1,
    Jolt,
    Nexus,
    Zisk,
    Bento,
}

impl Display for ProverId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProverId::Risc0 => write!(f, "risc0"),
            ProverId::SP1 => write!(f, "sp1"),
            ProverId::Jolt => write!(f, "jolt"),
            ProverId::Nexus => write!(f, "nexus"),
            ProverId::Zisk => write!(f, "zisk"),
            ProverId::Bento => write!(f, "bento"),
        }
    }
}

// /// Anc identifier used to select the hash function to evaluate.
// #[derive(ValueEnum, Clone, PartialEq)]
// pub enum HashFnId {
//     Sha256,
//     Poseidon,
//     Blake3,
//     Keccak256,
// }
//
// impl HashFnId {
//     /// Convert the identifier to a string.
//     pub fn to_string(&self) -> String {
//         match self {
//             HashFnId::Sha256 => "sha-256".to_string(),
//             HashFnId::Poseidon => "poseidon".to_string(),
//             HashFnId::Blake3 => "blake3".to_string(),
//             HashFnId::Keccak256 => "keccak256".to_string(),
//         }
//     }
// }

/// An identifier used to select the program to evaluate.
#[derive(ValueEnum, Clone, PartialEq)]
#[clap(rename_all = "kebab_case")]
pub enum ProgramId {
    Loop,
    Fibonacci,
    Tendermint,
    Reth,
    Raiko
}

impl Display for ProgramId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramId::Loop => write!(f, "loop"),
            ProgramId::Fibonacci => write!(f, "fibonacci"),
            ProgramId::Tendermint => write!(f, "tendermint"),
            ProgramId::Reth => write!(f, "reth"),
            ProgramId::Raiko => write!(f, "raiko"),
        }
    }
}
