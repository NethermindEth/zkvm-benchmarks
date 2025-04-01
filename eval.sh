#!/bin/bash
set -e
echo "Running $1, $2, $3, $4, $5"

# Function to check rust version and determine correct parameter name
check_rust_version() {
    local toolchain=$1
    local version_output

    if [ -z "$toolchain" ]; then
        version_output=$(rustc --version)
    else
        version_output=$(rustc +$toolchain --version)
    fi

    # Extract version number
    local version=$(echo "$version_output" | sed -E 's/rustc ([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
    local major=$(echo "$version" | cut -d. -f1)
    local minor=$(echo "$version" | cut -d. -f2)

    # Compare version with 1.81
    if [ "$major" -gt 1 ] || ([ "$major" -eq 1 ] && [ "$minor" -gt 81 ]); then
        echo "lower-atomic"  # New parameter name for Rust >= 1.81
    else
        echo "loweratomic"   # Old parameter name for Rust < 1.81
    fi
}

# If $2 == jolt, append precompiles to Cargo.toml
if [ "$2" = "jolt" ]; then
    cp Cargo.toml Cargo.toml.bak
    cat patches/jolt.txt >> Cargo.toml
fi

# Get program directory name as $1 and append "-$2" to it if $1 is "tendermint"
# or "reth"
if [ "$1" = "tendermint" ] || [ "$1" = "reth" ]; then
    program_directory="${1}-$2"
else
    program_directory="$1"
fi

echo "Building program"

# cd to program directory computed above
cd "benchmarks/$program_directory"

# If the prover is risc0, then build the program.
if [ "$2" == "risc0" ]; then
    echo "Building Risc0"
    # Use the risc0 toolchain.
    ATOMIC_PARAM=$(check_rust_version "risc0")
    CC_riscv32im_risc0_zkvm_elf=~/.risc0/cpp/bin/riscv32-unknown-elf-gcc \
      RUSTFLAGS="-C passes=$ATOMIC_PARAM -C link-arg=-Ttext=0x00200800 -C link-arg=--fatal-warnings -C panic=abort"\
      RISC0_FEATURE_bigint2=1\
      RUSTUP_TOOLCHAIN=risc0 \
      CARGO_BUILD_TARGET=riscv32im-risc0-zkvm-elf \
      cargo build --release --features $2

fi
# If the prover is sp1, then build the program.
if [ "$2" == "sp1" ]; then
    # The reason we don't just use `cargo prove build` from the SP1 CLI is we need to pass a --features ...
    # flag to select between sp1 and risc0.
    ATOMIC_PARAM=$(check_rust_version "succinct")
    RUSTFLAGS="-C passes=$ATOMIC_PARAM -C link-arg=-Ttext=0x00200800 -C panic=abort" \
        RUSTUP_TOOLCHAIN=succinct \
        CARGO_BUILD_TARGET=riscv32im-succinct-zkvm-elf \
        cargo build --release --ignore-rust-version --features $2
fi

if [ "$2" == "lita" ]; then
  echo "Building Lita"
  # Use the lita toolchain.
  CC_valida_unknown_baremetal_gnu="/valida-toolchain/bin/clang" \
    CFLAGS_valida_unknown_baremetal_gnu="--sysroot=/valida-toolchain -isystem /valida-toolchain/include" \
    RUSTUP_TOOLCHAIN=valida \
    CARGO_BUILD_TARGET=valida-unknown-baremetal-gnu \
    cargo build --release --ignore-rust-version --features $2

  # Lita does not have any hardware acceleration. Also it does not have an SDK
  # or a crate to be used on rust. We need to benchmark it without rust
  cd ../../
  ./eval_lita.sh $1 $2 $3 $program_directory $6
  exit
fi

if [ "$2" == "nexus" ]; then
  echo "Building Nexus"
  # Hardcode the memlimit to 8 MB
  RUSTFLAGS="-C link-arg=--defsym=MEMORY_LIMIT=0x80000 -C link-arg=-T../../nova.x" \
    CARGO_BUILD_TARGET=riscv32i-unknown-none-elf \
    RUSTUP_TOOLCHAIN=1.77.0 \
    cargo build --release --ignore-rust-version --features $2
fi

if [ "$2" == "zisk" ]; then
  echo "Building Zisk"
  cargo-zisk build --release --features $2
fi

cd ../../

echo "Running eval script"


# Check for AVX-512 support
if lscpu | grep -q avx512; then
  # If AVX-512 is supported, add the specific features to RUSTFLAGS
  export RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512ifma,+avx512vl"
else
  # If AVX-512 is not supported, just set target-cpu=native
  export RUSTFLAGS="-C target-cpu=native"
fi

# Set the logging level.
export RUST_LOG=info

# Detect whether we're on an instance with a GPU.
if nvidia-smi > /dev/null 2>&1; then
  FEATURES="$2, cuda"
  if [ "$2" = "jolt" ]; then
    export ICICLE_BACKEND_INSTALL_DIR=$(pwd)/target/release/deps/icicle/lib/backend
  fi
else
  FEATURES="$2"
fi

if [ "$2" = "jolt" ]; then
  export RUSTFLAGS=""
  export RUSTUP_TOOLCHAIN="nightly-2024-09-30"
fi


# Run the benchmark.
RISC0_INFO=1 \
    cargo run \
    -p zkvm-benchmarks-eval \
    --release \
    --no-default-features \
    --features "$FEATURES" \
    -- \
    --program "$1" \
    --prover "$2" \
    --shard-size "$3" \
    --filename "$4" \
     ${5:+$([[ "$1" == "fibonacci" ]] && echo "--fibonacci-input" || echo "--block-number") $5}
    # --hashfn "$3" \
    # --shard-size "$4" \
    # --filename "$5" \
    #  ${6:+$([[ "$1" == "fibonacci" ]] && echo "--fibonacci-input" || echo "--block-number") $6}

# Revert Cargo.toml as the last step
if [ "$2" = "jolt" ]; then
    mv Cargo.toml.bak Cargo.toml
fi

exit $?
