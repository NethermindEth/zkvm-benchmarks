#![no_main]
sp1_zkvm::entrypoint!(main);

use raiko_lib::{
    builder::calculate_block_header, input::GuestInput, proof_type::ProofType,
    protocol_instance::ProtocolInstance, CycleTracker,
};

pub mod sys;
pub use sys::*;

pub fn main() {
    let mut ct = CycleTracker::start("input");
    let input = sp1_zkvm::io::read_vec();
    let input = bincode::deserialize::<GuestInput>(&input).unwrap();
    ct.end();

    ct = CycleTracker::start("calculate_block_header");
    let header = calculate_block_header(&input);
    ct.end();

    ct = CycleTracker::start("ProtocolInstance");
    let pi = ProtocolInstance::new(&input, &header, ProofType::Sp1)
        .unwrap()
        .instance_hash();
    ct.end();

    sp1_zkvm::io::commit(&pi.0);
}
