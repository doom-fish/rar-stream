#!/usr/bin/env bash
# Profile-Guided Optimization benchmark script.
# Builds the vs_unrar benchmark with PGO instrumentation, runs it to collect
# profile data, then rebuilds with the profile applied.
#
# Usage: ./scripts/pgo-bench.sh [bench_filter]
# Example: ./scripts/pgo-bench.sh "matrix/iso-200_m5_128m/single"
set -euo pipefail

FILTER="${1:-matrix/iso-200_m5_128m/single}"
PROFDATA_DIR="target/pgo-profiles"
LLVM_PROFDATA="$(rustc --print target-libdir)/../bin/llvm-profdata"

if [ ! -f "$LLVM_PROFDATA" ]; then
  echo "Error: llvm-profdata not found at $LLVM_PROFDATA"
  echo "Install the llvm-tools component: rustup component add llvm-tools"
  exit 1
fi

echo "=== Step 1: Instrumented build ==="
rm -rf "$PROFDATA_DIR"
mkdir -p "$PROFDATA_DIR"
RUSTFLAGS="-C profile-generate=$PROFDATA_DIR" \
  cargo bench --bench vs_unrar --no-run 2>&1 | tail -3

echo ""
echo "=== Step 2: Collecting profile data ==="
RUSTFLAGS="-C profile-generate=$PROFDATA_DIR" \
  cargo bench --bench vs_unrar -- "$FILTER" 2>&1 | grep -E "time|thrpt" | head -4
echo "Profile data collected in $PROFDATA_DIR"

echo ""
echo "=== Step 3: Merging profiles ==="
"$LLVM_PROFDATA" merge -o "$PROFDATA_DIR/merged.profdata" "$PROFDATA_DIR"
echo "Merged to $PROFDATA_DIR/merged.profdata"

echo ""
echo "=== Step 4: PGO-optimized build ==="
RUSTFLAGS="-C profile-use=$PWD/$PROFDATA_DIR/merged.profdata -C llvm-args=-pgo-warn-missing-function" \
  cargo bench --bench vs_unrar --no-run 2>&1 | tail -3

echo ""
echo "=== Step 5: PGO benchmark ==="
RUSTFLAGS="-C profile-use=$PWD/$PROFDATA_DIR/merged.profdata" \
  cargo bench --bench vs_unrar -- "$FILTER" 2>&1 | grep -E "time|thrpt" | head -4

echo ""
echo "Done. Compare Step 2 (instrumented) vs Step 5 (PGO) for improvement."
