#!/bin/bash

# Build script for WASM frontend

set -e

echo "Building WASM frontend..."

# Check if wasm-bindgen is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Error: wasm-bindgen is not installed"
    echo "Install it with: cargo install wasm-bindgen-cli --version 0.2.106"
    exit 1
fi

# Check if wasm32-unknown-unknown target is added
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Navigate to frontend directory
cd frontend

# Build the WASM with cargo
echo "Building WASM with cargo..."
cargo build --release --target wasm32-unknown-unknown

# Generate JavaScript bindings
echo "Generating JavaScript bindings..."
cd ..
wasm-bindgen --target web --out-dir static/wasm frontend/target/wasm32-unknown-unknown/release/wptv_frontend.wasm

# Copy CSS
echo "Copying frontend assets..."
cp frontend/styles.css static/wasm/

# Generate index.html with correct WASM loader
cat > static/wasm/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Wazuh Process Tree Viewer</title>
    <link rel="stylesheet" href="/wasm/styles.css" />
</head>
<body>
    <div id="app"></div>
    <script type="module">
        import init from '/wasm/wptv_frontend.js';
        init();
    </script>
</body>
</html>
EOF

echo "WASM frontend built successfully!"
echo "Output is in static/wasm/"
