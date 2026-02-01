# PGO Profile Data

This directory contains Profile-Guided Optimization (PGO) data for building
optimized release binaries.

## Contents

- `merged.profdata.gz` - Compressed LLVM profile data generated from:
  - 300 iterations of LZSS decompression (8MB Alpine tar)
  - Representative of typical RAR4 workloads

## Usage

The profile data is used automatically during release builds via GitHub Actions.
For local PGO builds:

```bash
# Decompress profile data
gunzip -k pgo-data/merged.profdata.gz

# Build with PGO
RUSTFLAGS="-Cprofile-use=$(pwd)/pgo-data/merged.profdata" cargo build --release
```

## Regenerating Profiles

If you make significant changes to the decompression code, regenerate profiles:

```bash
# Build instrumented binary
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release --example profile

# Generate profile data
./target/release/examples/profile alpine-lzss 300

# Merge and compress
llvm-profdata merge -o pgo-data/merged.profdata /tmp/pgo-data/*.profraw
gzip pgo-data/merged.profdata
```

## Performance Impact

| Configuration | Speed | vs unrar |
|--------------|-------|----------|
| Without PGO | 295 MiB/s | 130% |
| With PGO | 311 MiB/s | 137% |

PGO provides ~5% additional speedup over the already optimized baseline.
