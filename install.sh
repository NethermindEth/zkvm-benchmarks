#!/usr/bin/env sh

# Exit immediately if a command exits with a non-zero status.
set -e

# Helper function to print an error message and exit.
error_exit() {
    echo "Error: $1 failed with exit code $?. Exiting."
    exit 1
}

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y || error_exit "Installing Rust"
source $HOME/.cargo/env

curl -L https://risczero.com/install | bash || error_exit "Installing Risc Zero toolchain"
export PATH="$PATH:$HOME/.risc0/bin"
rzup install || error_exit "Updating Risc Zero toolchain"
cargo risczero --version || error_exit "Checking cargo risczero version"

# Install the Succinct toolchain
curl -L https://sp1.succinct.xyz | bash || error_exit "Installing Succinct toolchain"
export PATH="$PATH:$HOME/.sp1/bin"
sp1up || error_exit "Updating Succinct toolchain"
cargo prove --version || error_exit "Checking cargo prove version"

# Install Lita toolchain
wget https://github.com/lita-xyz/llvm-valida-releases/releases/download/v0.7.0-alpha/llvm-valida-v0.7.0-alpha-linux-x86_64.tar.xz || error_exit "Downloading lita toolchain"
tar xf llvm-valida-v0.7.0-alpha-linux-x86_64.tar.xz || error_exit "Extracting lita toolchain"
cp install_lita_toolchain.sh valida-toolchain/install.sh
cd valida-toolchain
./install.sh || error_exit "Installing lita toolchain"
cd ../
rm llvm-valida-v0.7.0-alpha-linux-x86_64.tar.xz
rm -rf valida-toolchain

# Install the jolt toolchain
rustup toolchain install nightly-2024-09-30
cargo +nightly-2024-09-30 install --git https://github.com/a16z/jolt --force --bins jolt || error_exit "Installing jolt toolchain"

# Install Nexus toolchain
rustup target add riscv32i-unknown-none-elf
cargo install --git https://github.com/nexus-xyz/nexus-zkvm cargo-nexus --tag 'v0.2.4' || error_exit "Installing nexus toolchain"
cargo nexus --help || error_exit "Checking cargo nexus version"

# Install ZisK toolchain
curl https://raw.githubusercontent.com/0xPolygonHermez/zisk/main/ziskup/install.sh | bash || error_exit "Installing ZisK toolchain"

echo "All installations completed successfully."
