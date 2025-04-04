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

impl ProverId {
    /// Convert the identifier to a string.
    pub fn to_string(&self) -> String {
        match self {
            ProverId::Risc0 => "risc0".to_string(),
            ProverId::SP1 => "sp1".to_string(),
            ProverId::Jolt => "jolt".to_string(),
            ProverId::Nexus => "nexus".to_string(),
            ProverId::Zisk => "zisk".to_string(),
            ProverId::Bento => "bento".to_string(),
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
}

impl ProgramId {
    /// Convert the identifier to a string.
    pub fn to_string(&self) -> String {
        match self {
            ProgramId::Loop => "loop".to_string(),
            ProgramId::Fibonacci => "fibonacci".to_string(),
            ProgramId::Tendermint => "tendermint".to_string(),
            ProgramId::Reth => "reth".to_string(),
        }
    }
}
