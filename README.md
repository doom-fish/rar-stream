# rar-stream

> Fast RAR archive streaming for Node.js and browsers. Zero dependencies, pure Rust.

[![npm version](https://badge.fury.io/js/rar-stream.svg)](https://www.npmjs.com/package/rar-stream)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 **Fast**: Native Rust implementation with NAPI bindings
- 📦 **Zero dependencies**: No external runtime dependencies
- 🌐 **Cross-platform**: Works on Linux, macOS, Windows
- 🔄 **Streaming**: Stream files directly from RAR archives
- 📚 **Multi-volume**: Supports split archives (.rar, .r00, .r01, ...)
- 🗜️ **Full decompression**: LZSS, PPMd, and filters
- 🔐 **Encrypted archives**: AES-256 decryption (with `crypto` feature)
- 🆕 **RAR4 + RAR5**: Full support for both RAR formats
- 🌍 **Browser support**: WASM build available

## Installation

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
  
  // Or read a specific byte range (for streaming)
  const chunk = await file.createReadStream({ start: 0, end: 1023 });
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

### Stream Video from RAR (Partial Reads)

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

const media = new LocalFileMedia('./movie.rar');
const pkg = new RarFilesPackage([media]);
const files = await pkg.parse();

const video = files.find(f => f.name.endsWith('.mkv'));
if (video) {
  // Read first 1MB for header analysis
  const header = await video.createReadStream({ start: 0, end: 1024 * 1024 - 1 });
  console.log(`Video: ${video.name}, Total size: ${video.length} bytes`);
  
  // Stream in chunks
  const chunkSize = 1024 * 1024; // 1MB chunks
  for (let offset = 0; offset < video.length; offset += chunkSize) {
    const end = Math.min(offset + chunkSize - 1, video.length - 1);
    const chunk = await video.createReadStream({ start: offset, end });
    // Process chunk...
  }
}
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
  
  createReadStream(opts: { start: number; end: number }): Promise<Buffer>;
}
```

### RarFilesPackage

Parses single or multi-volume RAR archives.

```typescript
class RarFilesPackage {
  constructor(files: LocalFileMedia[]);
  
  parse(opts?: {
    maxFiles?: number;  // Limit number of files to parse
  }): Promise<InnerFile[]>;
}
```

### InnerFile

Represents a file inside the RAR archive.

```typescript
class InnerFile {
  readonly name: string;    // Full path inside archive
  readonly length: number;  // Uncompressed size in bytes
  
  readToEnd(): Promise<Buffer>;
  createReadStream(opts: { start: number; end: number }): Promise<Buffer>;
}
```

### Utility Functions

```typescript
// Check if buffer starts with RAR signature
function isRarArchive(buffer: Buffer): boolean;

// Parse RAR header from buffer (needs ~300 bytes)
function parseRarHeader(buffer: Buffer): RarFileInfo | null;

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
| Encrypted files | 🔜 | ✅ | `crypto` feature |
| Encrypted headers | — | 🔜 | Coming soon |
| Algorithm | AES-128-CBC | AES-256-CBC | — |
| Key derivation | SHA-1 based | PBKDF2-HMAC-SHA256 | — |

To enable encryption support, use the `crypto` feature:

```toml
[dependencies]
rar-stream = { version = "4", features = ["crypto"] }
```

## Performance

Benchmarks on M1 MacBook Pro (v4.x vs v3.x):

| Operation | rar-stream v4 (Rust) | rar-stream v3 (JS) |
|-----------|---------------------|-------------------|
| Parse 1GB archive | ~50ms | ~200ms |
| Decompress 100MB | ~800ms | ~3000ms |
| Memory usage | ~50MB | ~200MB |

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
