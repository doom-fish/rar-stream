<div align="center">
  <h1>rar-stream</h1>
</div>

<div align="center"><p>
    <a href="https://crates.io/crates/rar-stream"><img alt="Crates.io" src="https://img.shields.io/crates/v/rar-stream?style=for-the-badge&logo=rust&color=C9CBFF&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://crates.io/crates/rar-stream"><img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/rar-stream?style=for-the-badge&logo=rust&color=A6E3A1&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://docs.rs/rar-stream"><img alt="docs.rs" src="https://img.shields.io/docsrs/rar-stream?style=for-the-badge&logo=docs.rs&color=8bd5ca&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://www.npmjs.com/package/rar-stream"><img alt="npm" src="https://img.shields.io/npm/v/rar-stream?style=for-the-badge&logo=npm&color=F5A97F&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://www.npmjs.com/package/rar-stream"><img alt="npm Downloads" src="https://img.shields.io/npm/dm/rar-stream?style=for-the-badge&logo=npm&color=F5E0DC&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://github.com/doom-fish/rar-stream#license"><img alt="License" src="https://img.shields.io/crates/l/rar-stream?style=for-the-badge&logo=apache&color=ee999f&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://github.com/doom-fish/rar-stream/actions"><img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/doom-fish/rar-stream/ci.yml?branch=main&style=for-the-badge&logo=github&color=c69ff5&logoColor=D9E0EE&labelColor=302D41" /></a>
</p></div>

> Fast RAR archive streaming for Rust, Node.js, and browsers. Zero dependencies core.

## Installation

### Node.js

```bash
npm install rar-stream
```

Native binaries are automatically downloaded from [GitHub Releases](https://github.com/doom-fish/rar-stream/releases) during install. Supported platforms: Linux (x64, arm64), macOS (x64, arm64), Windows (x64).

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
| PPMd | 107 µs | 177 µs | 1.7x |
| Stored | 54 ns | 123 µs | 2170x |

Single-threaded decompression vs unrar across data types (200 MB, method 5, 128 MB dict):

| Data type | rar-stream | unrar | Ratio |
|-----------|-----------|-------|-------|
| Text | 631 MiB/s | 535 MiB/s | **1.18x faster** |
| Binary | 424 MiB/s | 439 MiB/s | 0.97x |
| ISO (x86) | 297 MiB/s | 440 MiB/s | 0.67x |

With the `parallel` feature (enabled by default in npm), rar-stream's pipeline beats unrar across all 24 benchmark scenarios. Best case: 1.9x faster.

| Archive | Size | Pipeline | unrar | Ratio |
|---------|------|----------|-------|-------|
| Binary (ISO) | 200 MB | 289ms | 420ms | 0.69x |
| Text | 200 MB | 119ms | 197ms | 0.61x |
| Mixed | 200 MB | 307ms | 524ms | 0.59x |
| Binary | 500 MB | 683ms | 1088ms | 0.63x |
| Text | 500 MB | 357ms | 590ms | 0.61x |
| Mixed | 1 GB | 1671ms | 2407ms | 0.69x |

<details>
<summary>Full benchmark matrix (24 scenarios)</summary>

```
Archive                  Single   Pipeline    Unrar   Pipe/Unrar
----------------------------------------------------------------
bin-500_m3_32m            1187ms       736ms    1122ms     0.66x
bin-500_m5_128m           1122ms       683ms    1088ms     0.63x
bin-500_m5_32m            1183ms       711ms    1119ms     0.64x
bin-500_m5_4m             1311ms       765ms    1206ms     0.63x
iso-200_m3_32m             693ms       289ms     420ms     0.69x
iso-200_m5_128m            694ms       296ms     455ms     0.65x
iso-200_m5_32m             697ms       290ms     418ms     0.69x
iso-200_m5_4m              694ms       293ms     426ms     0.69x
mixed-1g_m3_32m           2690ms      1818ms    2603ms     0.70x
mixed-1g_m5_128m          2909ms      1916ms    2852ms     0.67x
mixed-1g_m5_32m           2699ms      1794ms    2598ms     0.69x
mixed-1g_m5_4m            2611ms      1671ms    2407ms     0.69x
mixed-200_m3_32m           465ms       344ms     537ms     0.64x
mixed-200_m5_128m          413ms       307ms     524ms     0.59x
mixed-200_m5_32m           463ms       338ms     527ms     0.64x
mixed-200_m5_4m            487ms       350ms     531ms     0.66x
text-200_m3_32m            199ms       120ms     200ms     0.60x
text-200_m5_128m           196ms       119ms     230ms     0.52x
text-200_m5_32m            197ms       120ms     197ms     0.61x
text-200_m5_4m             218ms       127ms     199ms     0.64x
text-500_m3_32m            583ms       362ms     591ms     0.61x
text-500_m5_128m           572ms       365ms     628ms     0.58x
text-500_m5_32m            567ms       357ms     590ms     0.61x
text-500_m5_4m             623ms       382ms     620ms     0.62x
```

Ratio < 1.0 = rar-stream is faster.

</details>

## Migrating from v3.x

Drop-in replacement. Same API, native Rust implementation. Requires Node.js 18+.

## License

MIT
