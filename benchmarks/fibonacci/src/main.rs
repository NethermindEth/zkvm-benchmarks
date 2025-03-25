#![no_main]
#![cfg_attr(feature = "nexus", no_std)]

use core::hint::black_box;

#[cfg(feature = "risc0")]
risc0_zkvm::guest::entry!(main);

#[cfg(feature = "sp1")]
sp1_zkvm::entrypoint!(main);

#[cfg(feature = "lita")]
valida_rs::entrypoint!(main);

#[cfg(feature = "nexus")]
use nexus_rt::println;

#[cfg(feature = "zisk")]
ziskos::entrypoint!(main);

fn fibonacci(n: u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let sum = (a + b) % 7919; // Mod to avoid overflow
        a = b;
        b = sum;
    }
    b
}

#[cfg_attr(feature = "nexus", nexus_rt::main)]
pub fn main() {
    #[cfg(feature = "risc0")]
    let input: u32 = risc0_zkvm::guest::env::read();

    #[cfg(feature = "sp1")]
    let input: u32 = sp1_zkvm::io::read();

    #[cfg(feature = "lita")]
    let input = 300000;

    #[cfg(feature = "nexus")]
    let input = nexus_rt::read_private_input::<u32>().unwrap();

    #[cfg(feature = "ziskos")]
    let input = zisk_input();

    let result = black_box(fibonacci(black_box(input)));
    println!("result {}", result);
}

#[cfg(feature = "ziskos")]
fn zisk_input() -> u32 {
    let input: Vec<u8> = ziskos::read_input();
    u32::from_le_bytes(input.try_into().unwrap());
}
