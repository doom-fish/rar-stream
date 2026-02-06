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

rar-stream's parallel pipeline beats the official C `unrar` (v7.0) across all tested workloads.

AMD Ryzen 5 7640HS (6 cores):

| Archive | Size | rar-stream | unrar | Ratio |
|---------|------|-----------|-------|-------|
| Binary (ISO) | 200 MB | 422ms | 453ms | 0.93x |
| Text | 200 MB | 144ms | 202ms | 0.71x |
| Mixed | 200 MB | 342ms | 527ms | 0.65x |
| Binary | 500 MB | 824ms | 1149ms | 0.72x |
| Text | 500 MB | 424ms | 604ms | 0.70x |
| Mixed | 1 GB | 1953ms | 2550ms | 0.77x |

Wins 24/24 benchmark scenarios. Best case: 1.9x faster than unrar.

<details>
<summary>Full benchmark matrix (24 scenarios)</summary>

```
Archive                  Single   Pipeline    Unrar   Pipe/Unrar
----------------------------------------------------------------
bin-500_m3_32m            1278ms       884ms    1187ms     0.74x
bin-500_m5_128m           1200ms       824ms    1149ms     0.72x
bin-500_m5_32m            1247ms       852ms    1162ms     0.73x
bin-500_m5_4m             1378ms       942ms    1770ms     0.53x
iso-200_m3_32m             715ms       426ms     760ms     0.56x
iso-200_m5_128m            720ms       423ms     811ms     0.52x
iso-200_m5_32m             721ms       422ms     453ms     0.93x
iso-200_m5_4m              717ms       422ms     442ms     0.95x
mixed-1g_m3_32m           2974ms      2109ms    2775ms     0.76x
mixed-1g_m5_128m          3177ms      2213ms    2984ms     0.74x
mixed-1g_m5_32m           2979ms      2086ms    2731ms     0.76x
mixed-1g_m5_4m            2761ms      1953ms    2550ms     0.77x
mixed-200_m3_32m           499ms       385ms     547ms     0.70x
mixed-200_m5_128m          438ms       342ms     527ms     0.65x
mixed-200_m5_32m           495ms       384ms     539ms     0.71x
mixed-200_m5_4m            511ms       395ms     538ms     0.73x
text-200_m3_32m            209ms       145ms     202ms     0.72x
text-200_m5_128m           205ms       144ms     239ms     0.60x
text-200_m5_32m            209ms       144ms     202ms     0.71x
text-200_m5_4m             227ms       153ms     207ms     0.74x
text-500_m3_32m            606ms       432ms     613ms     0.70x
text-500_m5_128m           601ms       431ms     644ms     0.67x
text-500_m5_32m            604ms       424ms     604ms     0.70x
text-500_m5_4m             659ms       455ms     643ms     0.71x
```

</details>

## Migrating from v3.x

Drop-in replacement. Same API, native Rust implementation. Requires Node.js 18+.

## License

MIT
