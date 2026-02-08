# Contributing to rar-stream

Thanks for your interest in contributing! This guide covers setup, conventions, and the PR process.

## Getting Started

### Prerequisites

- Rust stable (1.70+)
- Node.js 18+
- For fuzz testing: Rust nightly (`rustup install nightly`)

### Setup

```bash
git clone https://github.com/doom-fish/rar-stream.git
cd rar-stream
npm install --ignore-scripts
npm run build:debug    # Fast debug build
cargo test --lib       # Rust unit tests
npm test               # Node.js integration tests
```

## Development Workflow

### Building

```bash
npm run build              # Release NAPI build
npm run build:debug        # Debug NAPI build (faster)
npm run build:wasm         # WASM build
```

### Testing

```bash
cargo test --lib                                         # Rust unit tests
cargo test --doc                                         # Doctests
npm test                                                 # All Node.js tests
npx vitest run rar-stream.test.ts -t "test name"         # Single test
cargo clippy --all-features -- -D warnings               # Lint
cargo fmt --check                                        # Format check
```

### Fuzz Testing

```bash
cargo +nightly fuzz list                                          # List targets
cargo +nightly fuzz run fuzz_decompress_rar5 -- -max_total_time=60  # Run 60s
```

### Unsafe Code Validation

```bash
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --lib -- decompress
```

## Conventions

- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `perf:`, `docs:`, `ci:`)
- **Unsafe code**: Allowed in hot paths, must have `// SAFETY:` comments
- **Clippy**: Pedantic lints enabled — see `Cargo.toml [lints.clippy]`
- **Doctests**: Add doctests for public API items when possible
- **Feature gates**: Use `#[cfg_attr(docsrs, doc(cfg(feature = "...")))]` on feature-gated items

## Pull Request Process

1. Fork the repo and create a branch from `main`
2. Make your changes with tests
3. Ensure CI passes: `cargo test --lib && cargo clippy --all-features -- -D warnings && npm test`
4. Open a PR with a clear description of the change
5. CI runs: lint, test (Rust + NAPI + WASM), Miri, fuzz smoke tests

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for an overview of the codebase structure and data flow.

## Test Fixtures

Test archives are in `__fixtures__/` organized by type:

- `single/` — Single RAR, one inner file
- `multi/` — Multi-volume RAR
- `compressed/` — LZSS, PPMd, delta, store
- `rar5/` — RAR5 format archives
- `sizes/` — 1B to 1MB for benchmarking

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
