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
- 🗜️ **Full decompression**: LZSS, PPMd, and VM filters
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

// Single RAR file
const media = new LocalFileMedia('./archive.rar');
const pkg = new RarFilesPackage([media]);

// Parse archive and get inner files
const files = await pkg.parse();

for (const file of files) {
  console.log(`${file.name}: ${file.length} bytes`);
  
  // Read entire file
  const buffer = await file.readToEnd();
  
  // Or read a specific range
  const chunk = await file.createReadStream({ start: 0, end: 1024 });
}
```

## API Reference

### LocalFileMedia

Represents a local RAR file.

```typescript
class LocalFileMedia {
  constructor(path: string);
  
  readonly name: string;
  readonly length: number;
  
  createReadStream(opts: { start: number; end: number }): Promise<Buffer>;
}
```

### RarFilesPackage

Parses single or multi-volume RAR archives.

```typescript
class RarFilesPackage {
  constructor(files: LocalFileMedia[]);
  
  parse(opts?: {
    maxFiles?: number;
  }): Promise<InnerFile[]>;
}
```

### InnerFile

Represents a file inside the RAR archive.

```typescript
class InnerFile {
  readonly name: string;
  readonly length: number;
  
  readToEnd(): Promise<Buffer>;
  createReadStream(opts: { start: number; end: number }): Promise<Buffer>;
}
```

### Utility Functions

```typescript
// Check if buffer contains RAR signature
function isRarArchive(buffer: Buffer): boolean;

// Parse RAR header from buffer (useful for detecting files)
function parseRarHeader(buffer: Buffer): RarFileInfo | null;
```

## Multi-Volume Archives

```javascript
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

// Load all volumes
const volumes = [
  new LocalFileMedia('./archive.rar'),
  new LocalFileMedia('./archive.r00'),
  new LocalFileMedia('./archive.r01'),
];

const pkg = new RarFilesPackage(volumes);
const files = await pkg.parse();

// Files spanning multiple volumes are handled automatically
const content = await files[0].readToEnd();
```

## Browser Usage (WASM)

```javascript
import init, { isRarArchive, WasmRarDecoder } from 'rar-stream/wasm';

await init();

// Check if data is a RAR archive
const buffer = new Uint8Array(/* ... */);
if (isRarArchive(buffer)) {
  // Create decoder
  const decoder = new WasmRarDecoder(unpackedSize);
  const decompressed = decoder.decompress(compressedData);
}
```

## Compression Support

| Method | Support | Description |
|--------|---------|-------------|
| Store (0x30) | ✅ | No compression |
| LZSS (0x31-0x35) | ✅ | Huffman + LZ77 |
| PPMd | ✅ | Context-based |
| VM Filters | ✅ | E8, Delta, Audio, RGB |

## Performance

Benchmarks on M1 MacBook Pro:

| Operation | rar-stream v2 (Rust) | rar-stream v1 (JS) |
|-----------|---------------------|-------------------|
| Parse 1GB archive | ~50ms | ~200ms |
| Decompress 100MB | ~800ms | ~3000ms |
| Memory usage | ~50MB | ~200MB |

## Migrating from v3.x

rar-stream v4.0 is a complete Rust rewrite with the same API. It's a drop-in replacement:

```javascript
// Works the same in v3.x and v4.x
import { LocalFileMedia, RarFilesPackage } from 'rar-stream';

const media = new LocalFileMedia('./archive.rar');
const pkg = new RarFilesPackage([media]);
const files = await pkg.parse();
```

### Breaking Changes

- Node.js 18+ required (was 14+)
- WASM module path changed to `rar-stream/wasm`
- Native Rust implementation (faster, lower memory)

## License

MIT © beam.cat

## Credits

- Based on [unrar](https://www.rarlab.com/) reference implementation
- PPMd algorithm by Dmitry Shkarin
