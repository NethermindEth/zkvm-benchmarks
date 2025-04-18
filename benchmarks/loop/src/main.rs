// This code is borrowed from RISC Zero's benchmarks.
//
// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]
#![cfg_attr(feature = "nexus", no_std)]

#[cfg(any(feature = "risc0", feature = "bento"))]
risc0_zkvm::guest::entry!(main);

#[cfg(feature = "sp1")]
sp1_zkvm::entrypoint!(main);

#[cfg(feature = "lita")]
valida_rs::entrypoint!(main);

#[cfg(target_os = "zkvm")]
use core::arch::asm;

#[cfg(feature = "zisk")]
ziskos::entrypoint!(main);

#[cfg_attr(feature = "nexus", nexus_rt::main)]
fn main() {
    let iterations = 3000 * 1024;
    for i in 0..iterations {
        memory_barrier(&i);
    }
}

#[allow(unused_variables)]
pub fn memory_barrier<T>(ptr: *const T) {
    #[cfg(target_os = "zkvm")]
    unsafe {
        asm!("/* {0} */", in(reg) (ptr))
    }
    #[cfg(not(target_os = "zkvm"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst)
}
