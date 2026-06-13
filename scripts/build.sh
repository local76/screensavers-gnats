#!/usr/bin/env bash
set -e
echo "Building release binary..."
cargo build --release
echo "Copying binary to dist/binaries/..."
mkdir -p dist/binaries
cp target/release/gnats dist/binaries/
echo "Done!"
