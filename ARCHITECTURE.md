# Architecture

## Overview

rar-stream is a RAR archive library written in Rust with bindings for Node.js (NAPI) and browsers (WASM). The core library has **zero dependencies** — all dependencies are optional via Cargo features.

## Layers

```text
┌─────────────────────────────────────────────────────────────┐
│  Bindings                                                   │
│  napi_bindings.rs (Node.js)  │  wasm_bindings.rs (Browser)  │
├─────────────────────────────────────────────────────────────┤
│  Orchestration (async feature)                              │
│  RarFilesPackage → InnerFile → FileMedia                    │
├─────────────────────────────────────────────────────────────┤
│  Parsing                                                    │
│  MarkerHeader → ArchiveHeader → FileHeader                  │
├─────────────────────────────────────────────────────────────┤
│  Decompression                                              │
│  Rar29Decoder (RAR4) │ Rar5Decoder (RAR5)                   │
├─────────────────────────────────────────────────────────────┤
│  Crypto (crypto feature)                                    │
│  Rar4Crypto (AES-128-CBC) │ Rar5Crypto (AES-256-CBC)        │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── lib.rs                  # Public API, re-exports, crate docs
├── parsing/                # RAR header parsing
│   ├── marker_header.rs    # RAR4/RAR5 signature detection
│   ├── rar4/               # RAR4 archive + file headers
│   └── rar5/               # RAR5 archive + file headers
├── decompress/             # Decompression engines
│   ├── mod.rs              # CompressionMethod enum, shared types
│   ├── rar29.rs            # RAR4 LZSS + Huffman decoder
│   ├── rar5/               # RAR5 LZSS + filters pipeline
│   ├── ppm/                # PPMd model (RAR4)
│   └── filters/            # VM filters (E8E9, delta, ARM, etc.)
├── crypto/                 # Encryption/decryption (crypto feature)
│   ├── rar4_crypto.rs      # AES-128-CBC, SHA-1 KDF
│   └── rar5_crypto.rs      # AES-256-CBC, PBKDF2-HMAC-SHA256
├── file_media.rs           # FileMedia trait + LocalFileMedia
├── rar_files_package.rs    # Multi-volume orchestration
├── rar_file_chunk.rs       # Byte ranges across volumes
├── inner_file.rs           # File inside archive, streaming reads
├── napi_bindings.rs        # Node.js NAPI exports
├── wasm_bindings.rs        # Browser WASM exports
├── formats/                # RAR format constants
├── error.rs                # RarError enum
└── crc32.rs                # CRC32 table (no-dep)
```

## Data Flow

### Parsing

```
Raw bytes → MarkerHeaderParser::detect_version()
         → ArchiveHeaderParser (RAR4 or RAR5)
         → FileHeaderParser → Vec<FileHeader>
```

### Reading a File

```
RarFilesPackage::new(volumes)
  → sort volumes (.rar → .r00 → .r01...)
  → parse headers across all volumes
  → build chunk map (which bytes in which volume)

InnerFile::read_to_end()
  → resolve chunks via binary search
  → FileMedia::read_range() for each chunk
  → decrypt (if encrypted, crypto feature)
  → decompress (if compressed)
  → return Vec<u8>
```

### Decompression Pipeline (RAR5, parallel feature)

```
Compressed blocks → Huffman decode → LZ match copy → Filter pipeline → Output
                                                      ├── E8E9 (x86)
                                                      ├── Delta
                                                      └── ARM
```

With the `parallel` feature, RAR5 decompression uses a producer-consumer pipeline:
- **Producer thread**: Huffman decoding + LZ match copy
- **Consumer thread**: Filter application + output assembly

## Feature Gates

```
default = []

async   = [tokio]               # Async file I/O, RarFilesPackage, InnerFile
crypto  = [aes, cbc, pbkdf2,    # AES encryption support
           sha2, sha1]
parallel = [rayon, crossbeam]   # Multi-threaded RAR5 decompression
napi    = [napi, napi-derive,   # Node.js bindings
           async, parallel]
wasm    = [wasm-bindgen, js-sys] # Browser bindings
```

## Multi-Volume Ordering

Volumes are sorted by extension:
1. `.rar` (main volume, always first)
2. `.r00`, `.r01`, `.r02`, ... (continuation volumes)
3. `.s00`, `.s01`, ... (overflow after `.r99`)

File data can span multiple volumes — the chunk map tracks byte ranges across volumes.

## Key Design Decisions

- **Range reads are inclusive**: `{ start: 0, end: 10 }` returns 11 bytes
- **Decompression is cached**: Once decompressed, the result is stored in the `InnerFile`
- **Binary search for seeking**: Chunk lookup is O(log n) for video streaming use cases
- **Zero-copy parsing**: Headers are parsed from borrowed byte slices, no unnecessary allocations
- **Unsafe in hot paths**: Performance-critical loops use unsafe with SAFETY comments, validated by Miri
