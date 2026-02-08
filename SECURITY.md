# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email **security@doom.fish** or use [GitHub's private vulnerability reporting](https://github.com/doom-fish/rar-stream/security/advisories/new)
3. Include steps to reproduce and any relevant details

We aim to respond within 48 hours and release a fix within 7 days for critical issues.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 5.x     | ✅ Active  |
| < 5.0   | ❌ EOL     |

## Security Considerations

### Encryption (`crypto` feature)

The `crypto` feature enables decryption of encrypted RAR archives:

- **RAR5**: AES-256-CBC with PBKDF2-HMAC-SHA256 key derivation
- **RAR4**: AES-128-CBC with SHA-1 key derivation

Passwords are held in memory only during parsing/decompression and are not persisted.

### Unsafe Code

This library uses `unsafe` in performance-critical decompression hot paths. All unsafe blocks:

- Have `// SAFETY:` comments explaining invariants
- Are validated by [Miri](https://github.com/rust-lang/miri) in CI on every push
- Are covered by 6 fuzz targets that run in CI

### Fuzzing

Continuous fuzzing with `cargo-fuzz` covers header parsing and decompression for both RAR4 and RAR5 formats. See `fuzz/` for targets.

### Dependencies

The core library has **zero dependencies**. Optional features add audited, well-known crates:

| Feature | Dependencies |
|---------|-------------|
| `async` | tokio |
| `crypto` | aes, cbc, pbkdf2, sha2, sha1 |
| `parallel` | rayon, crossbeam-channel |
| `napi` | napi, napi-derive |

`cargo audit` runs in CI on every push to check for known vulnerabilities.
