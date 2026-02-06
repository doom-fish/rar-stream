# rar-stream

> Fast RAR archive streaming for Rust, Node.js, and browsers. Zero dependencies core.

[![CI](https://github.com/doom-fish/rar-stream/actions/workflows/ci.yml/badge.svg)](https://github.com/doom-fish/rar-stream/actions/workflows/ci.yml)
[![npm version](https://badge.fury.io/js/rar-stream.svg)](https://www.npmjs.com/package/rar-stream)
[![npm downloads](https://img.shields.io/npm/dm/rar-stream.svg)](https://www.npmjs.com/package/rar-stream)
[![crates.io](https://img.shields.io/crates/v/rar-stream.svg)](https://crates.io/crates/rar-stream)
[![crates.io downloads](https://img.shields.io/crates/d/rar-stream.svg)](https://crates.io/crates/rar-stream)
[![docs.rs](https://docs.rs/rar-stream/badge.svg)](https://docs.rs/rar-stream)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What's New in v5.1.0

- ⚡ **Parallel RAR5 pipeline**: Beats the official C `unrar` in all 24 benchmark scenarios
- 🌐 **WASM RAR5 support**: Full RAR5 decompression in the browser via `WasmRar5Decoder`
- 🔌 **NAPI pipeline**: Node.js users get 40% faster decompression automatically
- 🧪 **E2E browser tests**: Full Playwright tests for upload → decompress → verify workflow

## Features

- 🚀 **Fast**: Native Rust implementation with NAPI bindings
- 📦 **Zero dependencies**: Core library has no external dependencies
- 🌐 **Cross-platform**: Works on Linux, macOS, Windows
- 🔄 **Streaming**: Stream files directly from RAR archives
- 📚 **Multi-volume**: Supports split archives (.rar, .r00, .r01, ...)
- 🗜️ **Full decompression**: LZSS, PPMd, and filters
- 🔐 **Encrypted archives**: AES-256/AES-128 decryption for RAR4 & RAR5
- 🆕 **RAR4 + RAR5**: Full support for both RAR formats
- 🌍 **Browser support**: WASM build available

## Installation

### Rust

```toml
[dependencies]
rar-stream = { version = "5", features = ["async", "crypto"] }
```

### Node.js

```bash
npm install rar-stream
# or
yarn add rar-stream
# or
pnpm add rar-stream
```

## Quick Start

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

// Open a RAR archive
const media = new LocalFileMedia('./archive.rar');
const pkg = new RarFilesPackage([media]);

// Parse and list inner files
const files = await pkg.parse();

for (const file of files) {
  console.log(`${file.name}: ${file.length} bytes`);
  
  // Read entire file into memory
  const buffer = await file.readToEnd();
  
  // Or stream (returns Node.js Readable)
  const stream = file.createReadStream();
  stream.pipe(process.stdout);
}
```

### Rust Quick Start

```rust
use rar_stream::{RarFilesPackage, ParseOptions, LocalFileMedia, FileMedia};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open a RAR archive
    let file: Arc<dyn FileMedia> = Arc::new(LocalFileMedia::new("archive.rar")?);
    let package = RarFilesPackage::new(vec![file]);

    // Parse and list files
    let files = package.parse(ParseOptions::default()).await?;
    for f in &files {
        println!("{}: {} bytes", f.name, f.length);
    }

    // Read file content
    let content = files[0].read_to_end().await?;
    println!("Read {} bytes", content.len());
    Ok(())
}
```

## Examples

### Extract a File to Disk

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';
import fs from 'fs';

const media = new LocalFileMedia('./archive.rar');
const pkg = new RarFilesPackage([media]);
const files = await pkg.parse();

// Find a specific file
const targetFile = files.find(f => f.name.endsWith('.txt'));
if (targetFile) {
  const content = await targetFile.readToEnd();
  fs.writeFileSync('./extracted.txt', content);
  console.log(`Extracted ${targetFile.name} (${content.length} bytes)`);
}
```

### Stream Video from RAR (Node.js Readable Stream)

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';
import fs from 'fs';

const media = new LocalFileMedia('./movie.rar');
const pkg = new RarFilesPackage([media]);
const files = await pkg.parse();

const video = files.find(f => f.name.endsWith('.mkv'));
if (video) {
  // Get a Node.js Readable stream for the entire file
  const stream = video.createReadStream();
  stream.pipe(fs.createWriteStream('./extracted-video.mkv'));
  
  // Or stream a specific byte range (for HTTP range requests)
  const rangeStream = video.createReadStream({ start: 0, end: 1024 * 1024 - 1 });
}
```

### WebTorrent Integration

Use `rar-stream` with WebTorrent to stream video from RAR archives inside torrents:

```javascript
import WebTorrent from 'webtorrent';
import { RarFilesPackage } from 'rar-stream';

const client = new WebTorrent();

client.add(magnetUri, (torrent) => {
  // Find RAR files (includes .rar, .r00, .r01, etc. for multi-volume)
  // WebTorrent files already implement the FileMedia interface!
  const rarFiles = torrent.files
    .filter(f => /\.(rar|r\d{2})$/i.test(f.name))
    .sort((a, b) => a.name.localeCompare(b.name));

  // No wrapper needed - pass torrent files directly
  const pkg = new RarFilesPackage(rarFiles);
  pkg.parse().then(innerFiles => {
    const video = innerFiles.find(f => f.name.endsWith('.mkv'));
    if (video) {
      // Stream video content - this returns a Node.js Readable
      const stream = video.createReadStream();
      
      // Pipe to HTTP response, media player, etc.
      stream.pipe(process.stdout);
    }
  });
});
```

### HTTP Range Request Handler (Express)

```javascript
import express from 'express';
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

const app = express();

// Pre-parse the RAR archive
const media = new LocalFileMedia('./videos.rar');
const pkg = new RarFilesPackage([media]);
const files = await pkg.parse();
const video = files.find(f => f.name.endsWith('.mp4'));

app.get('/video', (req, res) => {
  const range = req.headers.range;
  const fileSize = video.length;
  
  if (range) {
    const [startStr, endStr] = range.replace(/bytes=/, '').split('-');
    const start = parseInt(startStr, 10);
    const end = endStr ? parseInt(endStr, 10) : fileSize - 1;
    
    res.writeHead(206, {
      'Content-Range': `bytes ${start}-${end}/${fileSize}`,
      'Accept-Ranges': 'bytes',
      'Content-Length': end - start + 1,
      'Content-Type': 'video/mp4',
    });
    
    // Stream the range directly from the RAR archive
    const stream = video.createReadStream({ start, end });
    stream.pipe(res);
  } else {
    res.writeHead(200, {
      'Content-Length': fileSize,
      'Content-Type': 'video/mp4',
    });
    video.createReadStream().pipe(res);
  }
});

app.listen(3000);
```

### Multi-Volume Archives

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';
import fs from 'fs';
import path from 'path';

// Find all volumes in a directory
const dir = './my-archive';
const volumeFiles = fs.readdirSync(dir)
  .filter(f => /\.(rar|r\d{2})$/i.test(f))
  .sort()
  .map(f => new LocalFileMedia(path.join(dir, f)));

console.log(`Found ${volumeFiles.length} volumes`);

const pkg = new RarFilesPackage(volumeFiles);
const files = await pkg.parse();

// Files spanning multiple volumes are handled automatically
for (const file of files) {
  console.log(`${file.name}: ${file.length} bytes`);
}
```

### Check if a File is a RAR Archive

```javascript
import { isRarArchive, parseRarHeader } from 'rar-stream';
import fs from 'fs';

// Read first 300 bytes (enough for header detection)
const buffer = Buffer.alloc(300);
const fd = fs.openSync('./unknown-file', 'r');
fs.readSync(fd, buffer, 0, 300, 0);
fs.closeSync(fd);

if (isRarArchive(buffer)) {
  const info = parseRarHeader(buffer);
  if (info) {
    console.log(`First file: ${info.name}`);
    console.log(`Packed size: ${info.packedSize} bytes`);
    console.log(`Unpacked size: ${info.unpackedSize} bytes`);
    console.log(`Compression method: 0x${info.method.toString(16)}`);
  }
} else {
  console.log('Not a RAR archive');
}
```

### Limit Number of Files Parsed

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

const media = new LocalFileMedia('./large-archive.rar');
const pkg = new RarFilesPackage([media]);

// Only parse first 10 files (useful for previewing large archives)
const files = await pkg.parse({ maxFiles: 10 });
console.log(`Showing first ${files.length} files`);
```

## API Reference

### LocalFileMedia

Represents a local RAR file.

```typescript
class LocalFileMedia {
  constructor(path: string);
  
  readonly name: string;    // Filename (basename)
  readonly length: number;  // File size in bytes
  
  // Read a byte range into a Buffer
  // Create a Readable stream for a byte range
  createReadStream(opts: { start: number; end: number }): Readable;
}
```

### FileMedia Interface

Custom data sources (WebTorrent, S3, HTTP, etc.) must implement this interface:

```typescript
interface FileMedia {
  readonly name: string;
  readonly length: number;
  createReadStream(opts: { start: number; end: number }): Readable;
}
```

### RarFilesPackage

Parses single or multi-volume RAR archives.

```typescript
class RarFilesPackage {
  constructor(files: FileMedia[]);  // LocalFileMedia or custom FileMedia
  
  parse(opts?: {
    maxFiles?: number;  // Limit number of files to parse
  }): Promise<InnerFile[]>;
}
```

### InnerFile

Represents a file inside the RAR archive.

```typescript
import { Readable } from 'stream';

class InnerFile {
  readonly name: string;    // Full path inside archive
  readonly length: number;  // Uncompressed size in bytes
  
  // Read entire file into memory
  readToEnd(): Promise<Buffer>;
  
  // Create a Readable stream for the entire file or a byte range
  createReadStream(opts?: { 
    start?: number;   // Default: 0
    end?: number;     // Default: length - 1
  }): Readable;
}
```

### Utility Functions

```typescript
// Check if buffer starts with RAR signature
function isRarArchive(buffer: Buffer): boolean;

// Parse RAR header from buffer (needs ~300 bytes)
function parseRarHeader(buffer: Buffer): RarFileInfo | null;

// Convert a Readable stream to a Buffer
function streamToBuffer(stream: Readable): Promise<Buffer>;

// Create a FileMedia from any source with createReadStream
function createFileMedia(source: FileMedia): FileMedia;

interface RarFileInfo {
  name: string;
  packedSize: number;
  unpackedSize: number;
  method: number;
  continuesInNext: boolean;
}
```

## Compression Support

### RAR Format Compatibility

| Format | Signature | Support |
|--------|-----------|---------|
| RAR 1.5-4.x (RAR4) | `Rar!\x1a\x07\x00` | ✅ Full |
| RAR 5.0+ (RAR5) | `Rar!\x1a\x07\x01\x00` | ✅ Full |

### Compression Methods

| Method | RAR4 | RAR5 | Description |
|--------|------|------|-------------|
| Store | ✅ | ✅ | No compression |
| LZSS | ✅ | ✅ | Huffman + LZ77 sliding window |
| PPMd | ✅ | — | Context-based (RAR4 only) |

### Filter Support

| Filter | RAR4 | RAR5 | Description |
|--------|------|------|-------------|
| E8 | ✅ | ✅ | x86 CALL preprocessing |
| E8E9 | ✅ | ✅ | x86 CALL/JMP preprocessing |
| Delta | ✅ | ✅ | Byte delta per channel |
| ARM | — | ✅ | ARM branch preprocessing |
| Itanium | ✅ | — | IA-64 preprocessing |
| RGB | ✅ | — | Predictive color filter |
| Audio | ✅ | — | Audio sample predictor |

### Encryption Support

| Feature | RAR4 | RAR5 | Notes |
|---------|------|------|-------|
| Encrypted files | ✅ | ✅ | `crypto` feature |
| Encrypted headers | — | ✅ | RAR5 `-hp` archives |
| Algorithm | AES-128-CBC | AES-256-CBC | — |
| Key derivation | SHA-1 (262k rounds) | PBKDF2-HMAC-SHA256 | — |

To enable encryption support:

**Node.js/npm:** Encryption is always available.

**Rust:**
```toml
[dependencies]
rar-stream = { version = "5", features = ["async", "crypto"] }
```

## Performance

### RAR5 Decompression: Faster Than unrar

rar-stream's parallel pipeline **beats the official C `unrar`** (v7.0, multi-threaded) across all tested workloads.

Benchmark on AMD Ryzen 5 7640HS (6 cores):

| Archive | Size | rar-stream (pipeline) | unrar | Ratio |
|---------|------|----------------------|-------|-------|
| ISO (binary) | 200 MB | **422ms** | 453ms | **0.93×** |
| Text | 200 MB | **144ms** | 202ms | **0.71×** |
| Mixed | 200 MB | **342ms** | 527ms | **0.65×** |
| Binary | 500 MB | **824ms** | 1149ms | **0.72×** |
| Text | 500 MB | **424ms** | 604ms | **0.70×** |
| Mixed | 1 GB | **1953ms** | 2550ms | **0.77×** |

**Wins 24 out of 24 benchmark scenarios** across 6 data types × 4 compression settings.

Best case: **1.9× faster** than unrar (ISO 200MB, `-m5 -md128m`).

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

### Node.js (NAPI) Performance

| Configuration | Time (200MB) | vs unrar |
|---------------|-------------|----------|
| v5.0.0 (single-threaded) | 1127ms | 2.73× slower |
| **v5.1.0 (pipeline)** | **673ms** | **1.53× slower** |

The remaining NAPI gap vs native is I/O + buffer copy overhead, not decompression.

### Optimization Features

- **Parallel pipeline**: Decode + apply in parallel using rayon worker threads
- **Split-buffer decode**: Separates literals from commands for cache-friendly apply
- **LTO (Link-Time Optimization)**: Enabled by default in release builds
- **SIMD**: Automatic vectorization for E8/E9 filter scanning and memcpy
- **Zero-copy streaming**: Direct buffer access without intermediate copies

## Migrating from v3.x

rar-stream v4 is a complete Rust rewrite with the same API. It's a drop-in replacement:

```javascript
// Works the same in v3.x and v4.x
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

const media = new LocalFileMedia('./archive.rar');
const pkg = new RarFilesPackage([media]);
const files = await pkg.parse();
```

### Breaking Changes

- Node.js 18+ required (was 14+)
- Native Rust implementation (faster, lower memory)

## License

MIT

## Credits

- Based on [unrar](https://www.rarlab.com/) reference implementation
- PPMd algorithm by Dmitry Shkarin
