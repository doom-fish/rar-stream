#!/bin/bash
# Download Alpine ISO and create large test fixtures
#
# Creates RAR5 archives at various sizes for testing decompression
# at different file size boundaries (120MB, 300MB, 600MB, ~1GB)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
FIXTURES_DIR="$PROJECT_DIR/__fixtures__/large"
BENCHMARK_DIR="/tmp/rar-benchmark"

ALPINE_URL="https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/x86_64/alpine-extended-3.21.6-x86_64.iso"
ALPINE_ISO="$BENCHMARK_DIR/alpine.iso"
EXPECTED_SIZE=1093238784  # ~1GB extended ISO

mkdir -p "$BENCHMARK_DIR"
mkdir -p "$FIXTURES_DIR"

echo "=== Setting up large file test fixtures ==="

# Download Alpine ISO if not present
if [ ! -f "$ALPINE_ISO" ]; then
    echo ""
    echo "Downloading Alpine Linux ISO (~994MB)..."
    curl -L -o "$ALPINE_ISO" "$ALPINE_URL"
    
    # Verify size
    ACTUAL_SIZE=$(stat -c%s "$ALPINE_ISO" 2>/dev/null || stat -f%z "$ALPINE_ISO")
    if [ "$ACTUAL_SIZE" != "$EXPECTED_SIZE" ]; then
        echo "Warning: Downloaded size ($ACTUAL_SIZE) differs from expected ($EXPECTED_SIZE)"
    fi
else
    echo "Alpine ISO already exists: $ALPINE_ISO"
fi

# Create test files at specific sizes
create_test_file() {
    local size_mb=$1
    local size_bytes=$((size_mb * 1024 * 1024))
    local name="test_${size_mb}mb"
    local bin_file="$BENCHMARK_DIR/${name}.bin"
    local rar_file="$BENCHMARK_DIR/${name}_rar5.rar"
    
    if [ -f "$rar_file" ]; then
        echo "  $name: already exists"
        return
    fi
    
    echo "  Creating ${size_mb}MB test file..."
    head -c "$size_bytes" "$ALPINE_ISO" > "$bin_file"
    rar a -ep -m3 -ma5 "$rar_file" "$bin_file" >/dev/null 2>&1
    echo "  $name: done ($(ls -lh "$rar_file" | awk '{print $5}'))"
}

echo ""
echo "Creating test files from Alpine ISO..."
create_test_file 120
create_test_file 300
create_test_file 600

# Create full ISO archives
echo ""
echo "Creating full ISO RAR archives..."

if [ ! -f "$BENCHMARK_DIR/rar5-lzss.rar" ]; then
    echo "  Creating RAR5 LZSS (compressed)..."
    rar a -ep -m3 -ma5 "$BENCHMARK_DIR/rar5-lzss.rar" "$ALPINE_ISO" >/dev/null 2>&1
    echo "  rar5-lzss.rar: done"
else
    echo "  rar5-lzss.rar: already exists"
fi

if [ ! -f "$BENCHMARK_DIR/rar5-store.rar" ]; then
    echo "  Creating RAR5 Store (uncompressed)..."
    rar a -ep -m0 -ma5 "$BENCHMARK_DIR/rar5-store.rar" "$ALPINE_ISO" >/dev/null 2>&1
    echo "  rar5-store.rar: done"
else
    echo "  rar5-store.rar: already exists"
fi

echo ""
echo "=== Test fixtures ready ==="
echo ""
ls -lh "$BENCHMARK_DIR"/*.rar 2>/dev/null | head -20
echo ""
echo "Run tests with: npm test"
echo "Or run large tests only: npx vitest run -t 'Large'"
