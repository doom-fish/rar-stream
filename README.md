# rar-stream

> Fast RAR archive streaming for Rust, Node.js, and browsers. Zero dependencies core.

[![CI](https://github.com/doom-fish/rar-stream/actions/workflows/ci.yml/badge.svg)](https://github.com/doom-fish/rar-stream/actions/workflows/ci.yml)
[![npm version](https://badge.fury.io/js/rar-stream.svg)](https://www.npmjs.com/package/rar-stream)
[![crates.io](https://img.shields.io/crates/v/rar-stream.svg)](https://crates.io/crates/rar-stream)
[![docs.rs](https://docs.rs/rar-stream/badge.svg)](https://docs.rs/rar-stream)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

### Node.js

```bash
npm install rar-stream
```

### Rust

```toml
[dependencies]
rar-stream = { version = "5", features = ["async", "crypto"] }
```

### Browser (WASM)

```bash
wasm-pack build --target web --features "wasm,crypto" --no-default-features
```

## Quick Start

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

const media = new LocalFileMedia('./archive.rar');
const pkg = new RarFilesPackage([media]);
const files = await pkg.parse();

for (const file of files) {
  console.log(`${file.name}: ${file.length} bytes`);
  const buffer = await file.readToEnd();
  // or stream: file.createReadStream().pipe(process.stdout);
}
```

## Examples

Runnable examples in [`examples/`](./examples):

| Example | Run | Description |
|---------|-----|-------------|
| [basic.ts](./examples/basic.ts) | `npx tsx examples/basic.ts <path>` | Parse RAR, list files, read content |
| [http-stream.ts](./examples/http-stream.ts) | `npx tsx examples/http-stream.ts <rar>` | HTTP video server with range requests |
| [extract.rs](./examples/extract.rs) | `cargo run --release --example extract --features async -- archive.rar out/` | Extract files to disk |
| [browser.html](./examples/browser.html) | Open in browser (after `npm run build:wasm`) | WASM decompression demo |
| [profile.rs](./examples/profile.rs) | `cargo run --release --example profile` | Benchmark decompression loop |
| [benchmark_sizes.rs](./examples/benchmark_sizes.rs) | `cargo run --release --example benchmark_sizes` | Benchmark across file sizes |

## API

### Classes

| Class | Description |
|-------|-------------|
| `LocalFileMedia(path)` | Wraps a local file for reading |
| `RarFilesPackage(files)` | Parses single or multi-volume RAR archives |
| `InnerFile` | A file inside the archive |

### InnerFile

```typescript
class InnerFile {
  readonly name: string;    // Full path inside archive
  readonly length: number;  // Uncompressed size in bytes
  readToEnd(): Promise<Buffer>;
  createReadStream(opts?: { start?: number; end?: number }): Readable;
}
```

### FileMedia Interface

Custom data sources (WebTorrent, S3, HTTP) must implement:

```typescript
interface FileMedia {
  readonly name: string;
  readonly length: number;
  createReadStream(opts: { start: number; end: number }): Readable;
}
```

### Utility Functions

| Function | Description |
|----------|-------------|
| `isRarArchive(buffer)` | Check if buffer starts with RAR signature |
| `parseRarHeader(buffer)` | Parse RAR4 header from buffer (~300 bytes) |
| `streamToBuffer(stream)` | Convert Readable to Buffer |
| `createFileMedia(source)` | Wrap any `{ createReadStream }` as FileMedia |

## Format Support

| Feature | RAR4 | RAR5 |
|---------|------|------|
| Stored (no compression) | Yes | Yes |
| LZSS (Huffman + LZ77) | Yes | Yes |
| PPMd | Yes | -- |
| E8/E8E9 filters (x86) | Yes | Yes |
| Delta filter | Yes | Yes |
| ARM filter | -- | Yes |
| Itanium/RGB/Audio filters | Yes | -- |
| Encrypted files (AES) | Yes | Yes |
| Encrypted headers | -- | Yes |
| Multi-volume | Yes | Yes |

Encryption requires the `crypto` feature (always enabled in npm builds).

## Performance

Benchmarked against the official C `unrar` (v7.0) using `cargo bench`.

Core decompression (3 KB files, single-threaded):

| Method | rar-stream | unrar | Speedup |
|--------|-----------|-------|---------|
| LZSS | 8 µs | 92 µs | 11x |
| PPMd | 105 µs | 174 µs | 1.7x |
| Stored | 54 ns | 122 µs | 2257x |

Real archives (8 MB Alpine Linux ISO):

| Method | rar-stream | unrar | Speedup |
|--------|-----------|-------|---------|
| LZSS | 26 ms | 32 ms | 1.3x |
| PPMd | 26 ms | 32 ms | 1.2x |

<details>
<summary>Full benchmark matrix (24 scenarios, single-threaded)</summary>

```
Archive                  rar-stream     Unrar    Ratio
------------------------------------------------------
bin-500_m3_32m              1180ms     1152ms    1.02x
bin-500_m5_128m             1144ms     1122ms    1.02x
bin-500_m5_32m              1194ms     1140ms    1.05x
bin-500_m5_4m               1324ms     1233ms    1.07x
iso-200_m3_32m               690ms      435ms    1.58x
iso-200_m5_128m              680ms      460ms    1.48x
iso-200_m5_32m               685ms      436ms    1.57x
iso-200_m5_4m                680ms      431ms    1.58x
mixed-1g_m3_32m             2755ms     2673ms    1.03x
mixed-1g_m5_128m            2964ms     2875ms    1.03x
mixed-1g_m5_32m             2736ms     2635ms    1.04x
mixed-1g_m5_4m              2636ms     2440ms    1.08x
mixed-200_m3_32m             486ms      551ms    0.88x
mixed-200_m5_128m            433ms      535ms    0.81x
mixed-200_m5_32m             486ms      711ms    0.68x
mixed-200_m5_4m              633ms      612ms    1.03x
text-200_m3_32m              249ms      207ms    1.20x
text-200_m5_128m             231ms      236ms    0.98x
text-200_m5_32m              200ms      205ms    0.97x
text-200_m5_4m               218ms      197ms    1.11x
text-500_m3_32m              594ms      605ms    0.98x
text-500_m5_128m             581ms      643ms    0.90x
text-500_m5_32m              598ms      601ms    1.00x
text-500_m5_4m               639ms      628ms    1.02x
```

Ratio < 1.0 = rar-stream is faster. On large archives, single-threaded performance
is comparable to unrar. The `parallel` feature enables multi-threaded decompression
for additional speedup on multi-core systems.

</details>

## Migrating from v3.x

Drop-in replacement. Same API, native Rust implementation. Requires Node.js 18+.

## License

MIT
