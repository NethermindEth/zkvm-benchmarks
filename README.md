# ZKVM Benchmarks

A powerful benchmarking tool for ZKVM implementations.

This repository was originally forked from [zkvm-perf](https://github.com/succinctlabs/zkvm-perf/).

## Getting Started

### Prerequisites

You can run `install.sh` or do it manually

1. Install Rust:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup install nightly
   ```

2. Install the [SP1 toolchain](https://docs.succinct.xyz/getting-started/install.html):

   ```sh
   curl -L https://sp1.succinct.xyz | bash
   source ~/.bashrc
   sp1up
   cargo prove --version
   ```

3. Install the [Risc0 toolchain](https://dev.risczero.com/api/zkvm/install):

   ```sh
   curl -L https://risczero.com/install | bash
   source ~/.bashrc
   rzup install
   cargo risczero --version
   ```

4. Install the [Lita toolchain](https://lita.gitbook.io/lita-documentation/quick-start/installation-and-system-requirements):

``` sh
wget https://github.com/lita-xyz/llvm-valida-releases/releases/download/v0.7.0-alpha/llvm-valida-v0.7.0-alpha-linux-x86_64.tar.xz
tar xf llvm-valida-v0.7.0-alpha-linux-x86_64.tar.gz
cd valida-toolchain
sudo ./install.sh
```

5. Install the [Jolt toolchain](https://jolt.a16zcrypto.com/usage/install.html)

``` sh
rustup toolchain install nightly-2024-09-30
cargo +nightly-2024-09-30 install --git https://github.com/a16z/jolt --force --bins jolt || error_exit "Installing jolt toolchain"
```

6. Install the [Nexus toolchain](https://docs.nexus.xyz/zkvm/cli-quick-start)

``` sh
rustup target add riscv32i-unknown-none-elf
cargo install --git https://github.com/nexus-xyz/nexus-zkvm cargo-nexus --tag 'v0.2.4'
```

7. Install the [ZisK toolchain](https://0xpolygonhermez.github.io/zisk/getting_started/installation.html):

```sh
curl https://raw.githubusercontent.com/0xPolygonHermez/zisk/main/ziskup/install.sh | bash
```

8. Install [Docker](https://docs.docker.com/engine/install/ubuntu/).

**Note:** Run one round of a small program (e.g., Fibonacci) to download the R0 docker image before benchmarking to avoid affecting benchmark times.

**Note:** On Ubuntu 22.04, you might need to install libssl1.0 for the Risc0 toolchain. Follow these [instructions](https://stackoverflow.com/questions/72133316/libssl-so-1-1-cannot-open-shared-object-file-no-such-file-or-directory/73604364#73604364).

**Note:** For jolt you need to install `pkg-config` and `libssl-dev` (i.e. `apt
install pkg-config libssl-dev`)

**Note:** For ZisK you need to install additional packages on Ubuntu 22.04 or higher:
```sh
sudo apt-get install -y xz-utils jq curl build-essential qemu-system libomp-dev libgmp-dev nlohmann-json3-dev protobuf-compiler uuid-dev libgrpc++-dev libsecp256k1-dev libsodium-dev libpqxx-dev nasm libopenmpi-dev openmpi-bin openmpi-common
```

## Running Benchmarks

The main entry point for running the benchmarks is the `sweep.py` script. You can run it directly with Python:

```sh
python sweep.py [options]
```

Available options:

- `--filename`: Filename for the benchmark (default: "benchmark")
- `--trials`: Number of trials to run (default: 1)
- `--programs`: List of programs to benchmark (choices: loop, fibonacci, tendermint, reth, reth1, reth16, reth30)
- `--provers`: List of provers to use (choices: sp1, risc0, lita, jolt, nexus, zisk)
- `--shard-sizes`: List of shard sizes to use (default: [21])
- `--blocks`: List of block numbers for reth tests (default: ["22014900"] for reth1, ["17106222"] for reth16, ["19409768"] for reth30)
- `--fibonacci`: Inputs for the fibonacci benchmark (default: [100, 1000, 10000, 300000])

To run a single benchmark:

```sh
./eval.sh <program> <prover> <hashfn> <shard_size> <filename> [block_number]
```

### Example Command

```sh
python sweep.py --trials 5 --programs fibonacci --provers sp1 risc0 --shard-sizes 21 --fibonacci 100 1000 10000 300000
```

```sh
./eval.sh fibonacci sp1 poseidon 22 benchmark
./eval.sh fibonacci jolt-zkvm poseidon 22 benchmark
./eval.sh fibonacci risc0 poseidon 22 benchmark
./eval.sh reth sp1 poseidon 22 benchmark 19409768
```

## Analyzing Results

- Each benchmark run produces a CSV file with detailed performance metrics.
- Use the combined results file for a comprehensive view of all benchmarks.


## Contributing

We welcome contributions! Feel free to open issues or submit pull requests to help improve the benchmarks, add new features, or update documentation.

## License

This project is open source. Please see the LICENSE file for more details.

Happy benchmarking! 🚀
