#!/bin/bash

# Build script for WASM frontend

set -e

echo "Building WASM frontend..."

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "Error: trunk is not installed"
    echo "Install it with: cargo install trunk"
    exit 1
fi

# Check if wasm32-unknown-unknown target is added
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Navigate to frontend directory
cd frontend

# Build the WASM application
echo "Building with trunk..."
trunk build --release

echo "WASM frontend built successfully!"
echo "Output is in static/wasm/"
