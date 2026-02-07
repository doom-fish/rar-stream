# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [5.2.1] - 2026-02-07

### Bug Fixes

- Remove unused npm deps, fix all vulnerabilities

### Refactor

- Achieve true zero-dep core
- Remove async feature, inline tokio into napi


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

