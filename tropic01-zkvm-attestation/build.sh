#!/bin/bash
# Build and test zkVM attestation example

set -e

echo "════════════════════════════════════════════════════════"
echo "TROPIC01 zkVM Attestation - Build Script"
echo "════════════════════════════════════════════════════════"
echo ""

# Check if SP1 is installed
if ! command -v cargo-prove &> /dev/null; then
    echo "❌ SP1 toolchain not found"
    echo ""
    echo "Please install SP1:"
    echo "  curl -L https://sp1.succinct.xyz | bash"
    echo "  sp1up"
    echo "  cargo prove --version"
    exit 1
fi

echo "✅ SP1 toolchain found: $(cargo prove --version)"
echo ""

# Build guest program
echo "Building zkVM guest program..."
echo "(This may take a few minutes on first build)"
echo ""
cargo prove build

echo ""
echo "✅ Guest program built"
echo ""

# Build host program
echo "Building host program..."
cargo build --release

echo ""
echo "✅ Host program built"
echo ""

echo "════════════════════════════════════════════════════════"
echo "Build complete!"
echo "════════════════════════════════════════════════════════"
echo ""
echo "To run the example:"
echo "  cargo run --release -- /dev/ttyACM0 115200"
echo ""
echo "Or with logging:"
echo "  RUST_LOG=info cargo run --release -- /dev/ttyACM0 115200"
echo ""
