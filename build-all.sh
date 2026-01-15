#!/bin/bash

# Build script for complete application

set -e

echo "========================================"
echo "  Building Wazuh Process Tree Viewer"
echo "========================================"

# Step 1: Build WASM Frontend
echo ""
echo "Step 1/2: Building WASM Frontend..."
./build-frontend.sh

# Step 2: Build Backend
echo ""
echo "Step 2/2: Building Rust Backend..."
cargo build --release

echo ""
echo "========================================"
echo "  Build Complete!"
echo "========================================"
echo ""
echo "To run the application:"
echo "  1. Copy .env.example to .env and configure it"
echo "  2. Run: ./target/release/wazuh-process-tree-viewer"
echo ""
