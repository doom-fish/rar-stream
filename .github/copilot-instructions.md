# Copilot Instructions for rar-stream

## Build & Test Commands

```bash
# Build NAPI bindings (Node.js)
npm run build                    # Release build
npm run build:debug              # Debug build

# Build WASM (browser)
npm run build:wasm

# Run tests
npm test                         # All Node.js tests
npx vitest run rar-stream.test.ts -t "LZSS default"  # Single test
cargo test --lib                 # Rust unit tests only
npm run test:browser             # WASM/Playwright tests

# Lint
npm run lint                     # ESLint + Clippy
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

## Architecture

This is a Rust library with bindings for Node.js (NAPI) and browsers (WASM). The core library has **zero dependencies**; all dependencies are optional via Cargo features.

### Feature Flags (Cargo.toml)
- `napi` - Node.js bindings (implies `async`)
- `wasm` - Browser WASM bindings
- `async` - Async file reading with tokio

### Module Structure
```
src/
├── parsing/        # RAR header parsing (marker, archive, file headers)
├── decompress/     # Decompression: LZSS, PPMd, VM filters
├── formats/        # RAR format definitions
├── napi_bindings.rs    # Node.js NAPI exports
├── wasm_bindings.rs    # Browser WASM exports
├── rar_files_package.rs # Multi-volume archive orchestration
└── inner_file.rs   # Represents a file inside the archive
```

### Data Flow
1. `LocalFileMedia` wraps a file path and provides async range reads
2. `RarFilesPackage` takes multiple volumes, sorts them (.rar → .r00 → .r01...), parses headers
3. `InnerFile` represents extracted files, supports streaming via `createReadStream()`
4. Decompression runs on-demand when reading compressed content

## Conventions

- **No unsafe code**: `#![forbid(unsafe_code)]` is enforced
- **Clippy pedantic**: Most pedantic lints enabled (see `Cargo.toml [lints.clippy]`)
- **NAPI pattern**: Rust types prefixed with `Rust` internally, exposed as JS-friendly names (e.g., `RustInnerFile` → `InnerFile`)
- **Range reads are inclusive**: `{ start: 0, end: 10 }` returns 11 bytes
- **Multi-volume ordering**: Files are sorted by `.rar` first, then `.r00`, `.r01`, etc.

## Test Fixtures

Test archives are in `__fixtures__/` with patterns:
- `single/` - Single RAR, one inner file
- `multi/` - Multi-volume RAR, one inner file
- `single-splitted/` - Single RAR, multiple inner files
- `multi-splitted/` - Multi-volume RAR, multiple inner files
- `compressed/` - Various compression methods (store, LZSS, PPMd, delta filter)
