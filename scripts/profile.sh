#!/bin/bash
# Performance profiling workflow for rar-stream
#
# Prerequisites:
#   - cargo install flamegraph
#   - Linux: perf installed (sudo apt install linux-tools-generic)
#   - macOS: dtrace available (built-in)
#
# Usage:
#   ./scripts/profile.sh [command]
#
# Commands:
#   bench       Run benchmarks (default)
#   baseline    Save benchmark baseline
#   compare     Compare against baseline
#   flamegraph  Generate flamegraph SVG
#   perf        Run perf profiling (Linux only)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

case "${1:-bench}" in
    bench)
        echo "Running benchmarks..."
        cargo bench
        echo ""
        echo "Results saved to target/criterion/"
        echo "Open target/criterion/report/index.html for detailed reports"
        ;;
    
    baseline)
        echo "Saving benchmark baseline..."
        cargo bench -- --save-baseline main
        echo "Baseline saved. Use 'compare' to compare against it."
        ;;
    
    compare)
        echo "Comparing against baseline..."
        cargo bench -- --baseline main
        ;;
    
    flamegraph)
        echo "Generating flamegraph..."
        if ! command -v flamegraph &> /dev/null; then
            echo "Installing cargo-flamegraph..."
            cargo install flamegraph
        fi
        cargo flamegraph --bench decompress -o flamegraph.svg -- --bench
        echo "Flamegraph saved to flamegraph.svg"
        ;;
    
    perf)
        echo "Running perf profiling..."
        if [[ "$(uname)" != "Linux" ]]; then
            echo "perf is only available on Linux. Use 'flamegraph' instead."
            exit 1
        fi
        cargo build --release --bench decompress
        perf record -g ./target/release/deps/decompress-* --bench
        perf report
        ;;
    
    *)
        echo "Usage: $0 [bench|baseline|compare|flamegraph|perf]"
        exit 1
        ;;
esac
