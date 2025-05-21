#!/bin/bash
set -e
echo "Running $1, $2, $3, $4, $5, $6"

PROGRAM=$1;
PROVER=$2;
SHARD_SIZE=$3;
FILENAME=$4;
ADDED_ARGS=$5;
BLOCKS_DIR_SUFFIX=$6;

# Function to check rust version and determine correct parameter name
check_rust_version() {
    local toolchain=$PROGRAM
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

# If $PROVER == jolt or $PROGRAM" = raiko, append precompiles to Cargo.toml
if [ "$PROVER" = "jolt" ] || [ "$PROGRAM" = "raiko" ]; then
    cp Cargo.toml Cargo.toml.bak

    if [ "$PROGRAM" = "raiko" ]; then
      cp patches/raiko.txt Cargo.toml

      cp eval/Cargo.toml eval/Cargo.toml.bak
      cp patches/raikoeval.txt eval/Cargo.toml
    fi

    if [ "$PROVER" = "jolt" ]; then
      cat patches/jolt.txt >> Cargo.toml
    fi
fi

revert() {
  if [ "$PROVER" = "jolt" ] || [ "$PROGRAM" = "raiko" ]; then
      echo "Reverting Cargo.toml..."
      mv Cargo.toml.bak Cargo.toml 2>/dev/null || true

      if [ "$PROGRAM" = "raiko" ]; then
        mv eval/Cargo.toml.bak eval/Cargo.toml 2>/dev/null || true
      fi
  fi
}
trap revert EXIT

echo "Building program"

if [ "$PROGRAM" == "raiko" ]; then
    echo "Building Raiko for prover $PROVER"

    # Values from Raiko build script
    TOOLCHAIN_RISC0=nightly-2024-09-05
    TOOLCHAIN_SP1=nightly-2024-09-05

    # Run a builder inherited from Raiko itself
    if [ "$PROVER" == "sp1" ]; then
        RUSTUP_TOOLCHAIN=$TOOLCHAIN_SP1 \
            cargo run --bin raiko-sp1-builder
    elif [ "$PROVER" == "risc0" ]; then
        RUSTUP_TOOLCHAIN=$TOOLCHAIN_RISC0 \
            cargo run --bin raiko-risc0-builder
    else
        echo "Prover $PROVER is not supported for Raiko benchmark!"
        exit
    fi
else
  # Get program directory name as $PROGRAM and append "-$PROGRAM" to it if $PROGRAM is "tendermint"
  # or "reth"
  if [ "$PROGRAM" = "tendermint" ] || [ "$PROGRAM" = "reth" ]; then
      if [ "$PROVER" = "bento" ]; then
          program_directory="${1}-risc0" # Use risc0 directory for bento
      else
          program_directory="${1}-$PROVER"
      fi
  else
      program_directory="$PROGRAM"
  fi

  echo "Building program"

  # cd to program directory computed above
  cd "benchmarks/$program_directory"

  # If the prover is risc0 or bento, then build the program.
  if [ "$PROVER" == "risc0" ] || [ "$PROVER" == "bento" ]; then
      echo "Building Risc0"
      # Use the risc0 toolchain.
      ATOMIC_PARAM=$(check_rust_version "risc0")
      CC_riscv32im_risc0_zkvm_elf=~/.risc0/cpp/bin/riscv32-unknown-elf-gcc \
        RUSTFLAGS="-C passes=$ATOMIC_PARAM -C link-arg=-Ttext=0x00200800 -C link-arg=--fatal-warnings -C panic=abort"\
        RISC0_FEATURE_bigint2=1\
        RUSTUP_TOOLCHAIN=risc0 \
        CARGO_BUILD_TARGET=riscv32im-risc0-zkvm-elf \
        cargo build --release --features $PROVER

  # If the prover is sp1, then build the program.
  elif [ "$PROVER" == "sp1" ]; then
      # The reason we don't just use `cargo prove build` from the SP1 CLI is we need to pass a --features ...
      # flag to select between sp1 and risc0.
      ATOMIC_PARAM=$(check_rust_version "succinct")
      RUSTFLAGS="-C passes=$ATOMIC_PARAM -C link-arg=-Ttext=0x00200800 -C panic=abort" \
          RUSTUP_TOOLCHAIN=succinct \
          CARGO_BUILD_TARGET=riscv32im-succinct-zkvm-elf \
          cargo build --release --ignore-rust-version --features $PROVER
  elif [ "$PROVER" == "lita" ]; then
    echo "Building Lita"
    # Use the lita toolchain.
    CC_valida_unknown_baremetal_gnu="/valida-toolchain/bin/clang" \
      CFLAGS_valida_unknown_baremetal_gnu="--sysroot=/valida-toolchain -isystem /valida-toolchain/include" \
      RUSTUP_TOOLCHAIN=valida \
      CARGO_BUILD_TARGET=valida-unknown-baremetal-gnu \
      cargo build --release --ignore-rust-version --features $PROVER

    # Lita does not have any hardware acceleration. Also it does not have an SDK
    # or a crate to be used on rust. We need to benchmark it without rust
    cd ../../
    ./eval_lita.sh "$1" "$2" "$3" "$program_directory" "$5" # Pass potential extra arg $5
    exit
  elif [ "$PROVER" == "nexus" ]; then
    echo "Building Nexus"
    # Hardcode the memlimit to 8 MB
    RUSTFLAGS="-C link-arg=--defsym=MEMORY_LIMIT=0x80000 -C link-arg=-T../../nova.x" \
      CARGO_BUILD_TARGET=riscv32i-unknown-none-elf \
      RUSTUP_TOOLCHAIN=1.77.0 \
      cargo build --release --ignore-rust-version --features $PROVER
  elif [ "$PROVER" == "zisk" ]; then
    echo "Building Zisk"
    cargo-zisk build --release --features $PROVER
  fi

  cd ../../
fi

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
  FEATURES="$PROVER, cuda"
  if [ "$2" = "jolt" ]; then
    export ICICLE_BACKEND_INSTALL_DIR=$(pwd)/target/release/deps/icicle/lib/backend
  fi
else
  FEATURES="$PROVER"
fi

if [ "$PROVER" = "jolt" ]; then
  export RUSTFLAGS=""
  export RUSTUP_TOOLCHAIN="nightly-2024-09-30"
fi

# Prepare optional arguments for cargo run
cargo_run_opts=()
BENTO_URL_VALUE=""

# Determine Bento URL value if applicable
if [ "$2" = "bento" ]; then
  if [ "$#" -eq 6 ]; then # Case: extra_arg exists, bento_url is $6
    BENTO_URL_VALUE="$6"
  elif [ "$#" -eq 5 ]; then # Case: extra_arg is None, bento_url is $5
    BENTO_URL_VALUE="$5"
  fi
  if [ -z "$BENTO_URL_VALUE" ]; then
     echo "Error: Bento prover provided but URL parameter is missing or empty."
     exit 1
  fi
  cargo_run_opts+=(--bento-url "$BENTO_URL_VALUE")
fi

# Determine extra_arg parameter if applicable
# Check if $5 exists and is not the bento_url (which happens when $#=5 and prover=bento)
if [ "$#" -ge 5 ] && ! ([ "$#" -eq 5 ] && [ "$2" = "bento" ]); then
  EXTRA_ARG_VALUE="$5"
  if [ "$1" == "fibonacci" ]; then
    cargo_run_opts+=(--fibonacci-input "$EXTRA_ARG_VALUE")
  elif [ "$1" == "reth" ]; then
    cargo_run_opts+=(--block-number "$EXTRA_ARG_VALUE")
  fi
fi

# Run the benchmark.
RISC0_INFO=1 \
    cargo run \
    -p zkvm-benchmarks-eval \
    --release \
    --no-default-features \
    --features "$FEATURES" \
    -- \
    --program "$PROGRAM" \
    --prover "$PROVER" \
    --shard-size "$SHARD_SIZE" \
    --filename "$FILENAME" \
    "${cargo_run_opts[@]}" # Pass optional args safely
    --taiko-blocks-dir-suffix "$BLOCKS_DIR_SUFFIX"

# Revert Cargo.toml as the last step
if [ "$PROVER" = "jolt" ] || [ "$PROGRAM" = "raiko" ]; then
    mv Cargo.toml.bak Cargo.toml

    if [ "$PROGRAM" = "raiko" ]; then
      mv eval/Cargo.toml.bak eval/Cargo.toml
    fi
fi

exit $?
