# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [5.3.1] - 2026-02-13

### Bug Fixes

- Comprehensive security hardening, I/O optimization, and correctness fixes

### Documentation

- Add SECURITY.md, CONTRIBUTING.md, ARCHITECTURE.md, README TOC, coverage CI

### Ci

- Fix Miri cache corruption by using separate cache prefix
- Add cargo miri clean to prevent stale nightly cache artifacts


## [5.3.0] - 2026-02-08

### Bug Fixes

- Detect both RAR4 and RAR5 in isRarArchive, remove unused npm/ packages

### Documentation

- Complete docstrings and add doctests for docs.rs
- Restyle badges to match screencapturekit-rs (for-the-badge with custom colors)

### Features

- Add standalone async feature for pure-Rust async API


## [5.2.2] - 2026-02-08

### Ci

- Add E2E verification after publish


## [5.2.1] - 2026-02-08

### Bug Fixes

- Remove unused npm deps, fix all vulnerabilities
- Guard against bit buffer underflow with exhausted input
- Increase fuzz decompress per-execution timeout to 60s

### Miscellaneous Tasks

- Version sync, dead code cleanup, README perf update
- Feature-gate parallel code, add crypto/MSRV CI, remove dead peek16
- Add PGO benchmark script

### Performance

- Replace iter().position() with SSE2/SWAR byte search in E8E9 filter
- CRC32 slicing-by-8, LZSS bulk copy, zero-alloc VM filters
- Conditional BitReader refill, unrolled Huffman, skip-scan E8E9
- Bulk copy in PPMd alloc, RLE memset, pre-alloc table buffer
- Single-copy overlap match, inline filter check, cold VM path
- Extract byte_search to shared module, use SSE2 in RAR5 E8E9 filter
- Incremental consumed_bits tracking, cold slow Huffman path
- Branchless offset length bonus, streamlined read(), direct ptr E8E9
- Cold backref_error helper, reduce code size in hot decode loop
- Unsafe ptr access in delta/ARM filters, eliminate bounds checks
- Optimize HuffTable::build with memset and unchecked access
- Safe literal burst and simplified Huffman decode
- Over-allocate output buffer, 8-byte copy_match for short overlaps
- Hoist FastBits fields into register-locals in literal loop

### Refactor

- Achieve true zero-dep core
- Remove async feature, inline tokio into napi

### Ci

- Restructure build.yml for per-platform npm publishing


## [5.2.0] - 2026-02-07

### Bug Fixes

- Cargo fmt and eslint unused var

### Features

- *(wasm)* Add extract_file() for one-call decompression
- *(wasm)* Add WasmRarArchive with entries() and extract() API

### Refactor

- *(wasm)* Align browser API naming with NAPI


## [5.1.1] - 2026-02-07

### Bug Fixes

- Address 9 issues from deep code review
- Prevent subtract overflow in VM read_data with crafted RAR4 input
- *(ci)* Exclude large alpine test from Miri to prevent timeout
- Harden BitDecoder, FastBits and copy_bytes against buffer overreads and arithmetic overflow

### Documentation

- Streamline README, remove emojis, add examples table
- Update copilot instructions with release flow, fuzz, miri
- Update benchmarks with current numbers
- Update benchmarks with parallel pipeline results

### Performance

- Use doubling copy for overlapping backreferences

### Refactor

- Rewrite examples as TypeScript, Rust, and WASM

### Styling

- Fix formatting

### Testing

- Add fuzz testing and Miri CI


## [5.1.0] - 2026-02-06

### Performance

- Parallel multi-threaded RAR5 decoding pipeline with split-buffer decode
- Aggressive inlining in hot decode paths (Huffman, BitReader, copy_match)
- SIMD memchr for E8/E8E9 filter byte search
- Bulk copies in flush_to_output and VM filter
- Optimized Huffman rebuild and BitReader
- LTO and optimized release profile

### Features

- WASM RAR5 decompression and `dataOffset` in header parsers
- Parallel feature flag for multi-threaded decoding

### Bug Fixes

- PPMd EXP_ESCAPE and ns2_bs_indx out-of-bounds access
- Clippy compliance with SAFETY comments on all unsafe blocks
- Cargo publish ordering: crates.io publishes before npm

### Testing

- E2E Playwright browser tests (RAR4/RAR5 upload → decompress → verify)
- RAR5 browser decompression tests
- Parallel feature unit tests

## [5.0.1] - 2026-02-06

### Miscellaneous Tasks

- Update Cargo.toml dependencies


## [5.0.0] - 2026-01-31

### Bug Fixes

- Include crypto feature in WASM build

### Styling

- Cargo fmt


### Added

- **RAR5 format support** (Issue #35)
  - Full parsing of RAR5 archives (signature `Rar!\x1a\x07\x01\x00`)
  - RAR5 stored file extraction
  - RAR5 compressed file decompression (LZSS-based)
  - RAR5 filters: Delta, E8, E8E9, ARM
  - RAR5 multi-volume archive support
- **Encrypted archive support** (optional `crypto` feature)
  - RAR5 encryption: AES-256-CBC with PBKDF2-HMAC-SHA256 key derivation
  - Password verification via 64-bit check value
  - Full decryption of stored and compressed files
- CI now tests against stable Rust

### Changed

- Block decoder now correctly handles RAR5 symbol encoding:
  - Symbol 256 = Filter command
  - Symbol 257 = Repeat last length
  - Symbols 262+ = New offset with length slot

### Fixed

- RAR5 table length repeat codes (2*num formula)
- RAR5 offset decoding for small slots (<4)
- Extra area field size parsing

