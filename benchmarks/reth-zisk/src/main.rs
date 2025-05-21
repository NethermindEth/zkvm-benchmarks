//! An implementation of a type-1, bytecompatible compatible, zkEVM written in Rust & SP1.
//!
//! The flow for the guest program is based on Zeth and rsp.
//!
//! Reference: https://github.com/risc0/zeth
//!            https://github.com/succinctlabs/rsp

#![no_main]
ziskos::entrypoint!(main);

use ziskos::read_input;

use rsp_client_executor::{io::ClientExecutorInput, ClientExecutor, EthereumVariant};

fn main() {
    // Read the input.
    let input = read_input();
    let block = bincode::deserialize::<ClientExecutorInput>(&input).unwrap();

    // Execute the block.
    let header = ClientExecutor
        .execute::<EthereumVariant>(block)
        .expect("failed to execute client");
    let block_hash = header.hash_slow();

    println!("block_hash: {:?}", block_hash);
}
