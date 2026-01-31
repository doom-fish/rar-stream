# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- **RAR5 format support** (Issue #35)
  - Full parsing of RAR5 archives (signature `Rar!\x1a\x07\x01\x00`)
  - RAR5 stored file extraction
  - RAR5 compressed file decompression (LZSS-based)
  - RAR5 filters: Delta, E8, E8E9, ARM
  - RAR5 multi-volume archive support
- CI now tests against stable Rust

### Changed

- Block decoder now correctly handles RAR5 symbol encoding:
  - Symbol 256 = Filter command
  - Symbol 257 = Repeat last length
  - Symbols 262+ = New offset with length slot

### Fixed

- RAR5 table length repeat codes (2*num formula)
- RAR5 offset decoding for small slots (<4)

