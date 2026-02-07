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

# Fuzz testing (requires nightly)
cargo +nightly fuzz run fuzz_decompress_rar5 -- -max_total_time=60
cargo +nightly fuzz list          # List all targets

# Miri (unsafe UB detection, requires nightly)
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --lib -- decompress
```

## Release Process

Releases use **release-plz** for changelog/version PRs and **build.yml** for publishing.

### Automated flow (preferred)

1. Push commits to `main` using conventional commits (`feat:`, `fix:`, `perf:`, etc.)
2. **release-plz** (`.github/workflows/release-plz.yml`) runs on every push to main:
   - Compares local source against published crate on crates.io
   - If changes exist, creates a PR with version bump + CHANGELOG update
   - Config: `release-plz.toml` — `publish = false`, `git_tag_enable = false` (PR only, no auto-publish)
3. **Merge the release-plz PR** — this bumps version in Cargo.toml and updates CHANGELOG.md
4. **Create and push a git tag**: `git tag v5.x.x && git push origin v5.x.x`
5. **build.yml** triggers on `v*` tags and:
   - Builds native binaries for 6 platforms (linux-x64, linux-arm64, darwin-x64, darwin-arm64, windows-x64, linux-musl)
   - Publishes to **crates.io** first (fails the job if it fails)
   - Publishes to **npm** second (OIDC trusted publishing, `--provenance`)
   - Creates **GitHub Release** with auto-generated notes

### Manual release (when release-plz bump is wrong)

1. Bump version in **both** `Cargo.toml` and `package.json` (must stay in sync)
2. Update `CHANGELOG.md`
3. Commit, tag, push: `git tag v5.x.x && git push origin v5.x.x`
4. build.yml handles the rest

### Important notes

- **Version sync**: `package.json` and `Cargo.toml` versions must always match
- **Cargo.toml `[package] exclude`**: Keeps crate under 10MB (crates.io limit). Update if adding new top-level dirs
- **`--allow-dirty`**: Required in build.yml because CI checkout has `.node` build artifacts
- **release-plz** only creates PRs — it never publishes or tags (configured in `release-plz.toml`)
- **Conventional commits** drive version bumps: `feat:` → minor, `fix:` → patch, `BREAKING CHANGE` → major

## Architecture

This is a Rust library with bindings for Node.js (NAPI) and browsers (WASM). The core library has **zero dependencies**; all dependencies are optional via Cargo features.

### Feature Flags (Cargo.toml)
- `napi` - Node.js bindings (implies `async` + `parallel`)
- `wasm` - Browser WASM bindings
- `async` - Async file reading with tokio
- `crypto` - AES encryption support (RAR4 + RAR5)
- `parallel` - Multi-threaded RAR5 decompression (rayon)

### Module Structure
```
src/
├── parsing/        # RAR header parsing (marker, archive, file headers)
├── decompress/     # Decompression: LZSS, PPMd, VM filters
├── formats/        # RAR format definitions
├── crypto/         # AES decryption (RAR4: AES-128, RAR5: AES-256)
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

## CI Workflows

- **ci.yml**: Runs on push/PR to main — lint, test Rust/NAPI/WASM, Miri, fuzz smoke tests
- **release-plz.yml**: Runs on push to main — creates release PRs with changelog
- **build.yml**: Runs on `v*` tags — cross-compile, publish crates.io + npm + GitHub Release

## Conventions

- **Unsafe code**: `unsafe_code = "allow"` — used in decompression hot paths with SAFETY comments. Validated by Miri in CI
- **Clippy pedantic**: Most pedantic lints enabled (see `Cargo.toml [lints.clippy]`)
- **NAPI pattern**: Rust types prefixed with `Rust` internally, exposed as JS-friendly names (e.g., `RustInnerFile` → `InnerFile`)
- **Range reads are inclusive**: `{ start: 0, end: 10 }` returns 11 bytes
- **Multi-volume ordering**: Files are sorted by `.rar` first, then `.r00`, `.r01`, etc.
- **Version sync**: `package.json` and `Cargo.toml` versions must match
- **createReadStream** returns `Promise<Buffer>` (not a Node.js stream)

## Test Fixtures

Test archives are in `__fixtures__/` with patterns:
- `single/` - Single RAR, one inner file
- `multi/` - Multi-volume RAR, one inner file
- `single-splitted/` - Single RAR, multiple inner files
- `multi-splitted/` - Multi-volume RAR, multiple inner files
- `compressed/` - Various compression methods (store, LZSS, PPMd, delta filter)
- `rar5/` - RAR5 format archives (stored, compressed)
- `sizes/` - Various file sizes for benchmarking (1B to 1MB)

## Fuzz Testing

6 fuzz targets in `fuzz/fuzz_targets/`:
- `fuzz_parse_rar4`, `fuzz_parse_rar5` — header parsing
- `fuzz_decompress_rar4`, `fuzz_decompress_rar5` — raw decompression
- `fuzz_archive_rar4`, `fuzz_archive_rar5` — full parse-then-decompress pipeline

Seed corpus in `fuzz/corpus/` from test fixtures. Run extended fuzzing locally:
```bash
cargo +nightly fuzz run fuzz_decompress_rar5 -- -max_total_time=3600  # 1 hour
```
