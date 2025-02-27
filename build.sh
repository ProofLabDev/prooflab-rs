#!/bin/bash
set -e

echo "Building all crates in workspace..."
cargo build --all

echo "Running tests..."
cargo test --all

echo "Building release version..."
cargo build --release -p prooflab

echo "Build completed successfully!"
echo "You can find the prooflab executable at: ./target/release/prooflab"