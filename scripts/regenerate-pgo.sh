#!/bin/bash
# Regenerate PGO profile data
# Run this after significant changes to decompression code

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PGO_DIR="$PROJECT_DIR/pgo-data"
TMP_DIR="/tmp/pgo-data-$$"

echo "=== Regenerating PGO Profile Data ==="

# Clean up
rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR"

# Step 1: Build instrumented binary
echo ""
echo "Step 1: Building instrumented binary..."
cd "$PROJECT_DIR"
RUSTFLAGS="-Cprofile-generate=$TMP_DIR" cargo build --release --example profile

# Step 2: Generate profile data
echo ""
echo "Step 2: Generating profile data..."
./target/release/examples/profile alpine-lzss 300

# Step 3: Merge profiles
echo ""
echo "Step 3: Merging profiles..."

# Find llvm-profdata (try different versions)
PROFDATA=""
for ver in 21 20 19 18 17 16; do
    if command -v "llvm-profdata-$ver" &> /dev/null; then
        PROFDATA="llvm-profdata-$ver"
        break
    fi
done

if [ -z "$PROFDATA" ] && command -v llvm-profdata &> /dev/null; then
    PROFDATA="llvm-profdata"
fi

if [ -z "$PROFDATA" ]; then
    echo "Error: llvm-profdata not found"
    exit 1
fi

echo "Using: $PROFDATA"
$PROFDATA merge -o "$PGO_DIR/merged.profdata" "$TMP_DIR"/*.profraw

# Step 4: Compress
echo ""
echo "Step 4: Compressing..."
gzip -f "$PGO_DIR/merged.profdata"

# Clean up
rm -rf "$TMP_DIR"

echo ""
echo "=== Done ==="
ls -lh "$PGO_DIR"

# Step 5: Verify with a test build
echo ""
echo "Step 5: Verifying with PGO build..."
gunzip -k "$PGO_DIR/merged.profdata.gz"
RUSTFLAGS="-Cprofile-use=$PGO_DIR/merged.profdata" cargo build --release --example profile
./target/release/examples/profile alpine-lzss 50
rm "$PGO_DIR/merged.profdata"

echo ""
echo "PGO data regenerated successfully!"
