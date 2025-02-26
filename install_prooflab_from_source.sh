#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

echo "Installing prooflab-rs from source..."

BASE_DIR=$HOME
PROOFLAB_DIR="${PROOFLAB_DIR-"$BASE_DIR/.prooflab"}"
PROOFLAB_BIN_DIR="$PROOFLAB_DIR/bin"
PROOFLAB_BIN_PATH="$PROOFLAB_BIN_DIR/prooflab-rs"

# Create bin directory
mkdir -p "$PROOFLAB_BIN_DIR"

# Build from source using local code
echo "Building prooflab-rs from source..."
cargo build --release
cp target/release/prooflab-rs "$PROOFLAB_BIN_PATH"

chmod +x "$PROOFLAB_BIN_PATH"

# Store the correct profile file
case $SHELL in
*/zsh)
    PROFILE="${ZDOTDIR-"$HOME"}/.zshenv"
    PREF_SHELL=zsh
    ;;
*/bash)
    PROFILE=$HOME/.bashrc
    PREF_SHELL=bash
    ;;
*/fish)
    PROFILE=$HOME/.config/fish/config.fish
    PREF_SHELL=fish
    ;;
*/ash)
    PROFILE=$HOME/.profile
    PREF_SHELL=ash
    ;;
*)
    echo "prooflab-rs: could not detect shell, manually add ${PROOFLAB_BIN_DIR} to your PATH."
    exit 1
esac

# Only add to PATH if it isn't already there
if [[ ":$PATH:" != *":${PROOFLAB_BIN_DIR}:"* ]]; then
    if [[ "$PREF_SHELL" == "fish" ]]; then
        echo >> "$PROFILE" && echo "fish_add_path -a $PROOFLAB_BIN_DIR" >> "$PROFILE"
    else
        echo >> "$PROFILE" && echo "export PATH=\"\$PATH:$PROOFLAB_BIN_DIR\"" >> "$PROFILE"
    fi
fi

echo "prooflab-rs built and installed successfully in $PROOFLAB_BIN_PATH"
echo "Detected your preferred shell is $PREF_SHELL and added to PATH."
echo "Installing zkVM toolchains"

# Check for RISC0 toolchain
echo "Checking for RISC0 toolchain..."
if ! command -v rzup &> /dev/null; then
    echo "Installing RISC0 toolchain..."
    curl -L https://risczero.com/install | bash
    export PATH="$PATH:$HOME/.risc0/bin"
    rzup install
else
    echo "RISC0 toolchain already installed"
fi
cargo risczero --version

# Check for SP1 toolchain
echo "Checking for SP1 toolchain..."
if ! command -v sp1up &> /dev/null; then
    echo "Installing SP1 toolchain..."
    curl -L https://sp1.succinct.xyz | bash
    export PATH="$PATH:$HOME/.sp1/bin"
    sp1up -v v4.0.1
else
    echo "SP1 toolchain already installed"
fi
cargo prove --version

# Set up workspaces directory
echo "Setting up workspaces..."
mkdir -p "$PROOFLAB_DIR/workspaces"
cp -r "$SCRIPT_DIR/workspaces/"* "$PROOFLAB_DIR/workspaces/"

echo "Run 'source $PROFILE' or start a new terminal session to use prooflab-rs!" 