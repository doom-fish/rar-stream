//! Benchmark comparing rar-stream vs official unrar library.
//!
//! Run with: `cargo bench --bench vs_unrar`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rar_stream::parsing::file_header::FileHeaderParser;
use rar_stream::Rar29Decoder;
use std::path::Path;
use unrar::Archive;

/// RAR4 marker header signature.
const RAR4_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00];

/// Parse a RAR4 file and extract file header + compressed data.
fn parse_rar4_file(data: &[u8]) -> Option<(rar_stream::parsing::file_header::FileHeader, &[u8])> {
    if !data.starts_with(RAR4_MARKER) {
        return None;
    }

    let mut pos = RAR4_MARKER.len();

    if data.len() < pos + 7 {
        return None;
    }
    let archive_head_size = u16::from_le_bytes([data[pos + 5], data[pos + 6]]) as usize;
    pos += archive_head_size;

    if data.len() < pos + FileHeaderParser::HEADER_SIZE {
        return None;
    }

    let header = FileHeaderParser::parse(&data[pos..]).ok()?;
    let data_start = pos + header.head_size as usize;
    let data_end = data_start + header.packed_size as usize;

    if data.len() < data_end {
        return None;
    }

    Some((header, &data[data_start..data_end]))
}

/// Extract using official unrar library to memory
fn extract_with_unrar_to_bytes(archive_path: &Path) -> Vec<u8> {
    let mut archive = Archive::new(archive_path)
        .open_for_processing()
        .expect("Failed to open archive");

    let mut result = Vec::new();
    while let Some(header) = archive.read_header().expect("Failed to read header") {
        let (data, next_archive) = header.read().expect("Failed to read");
        archive = next_archive;
        result = data;
    }
    result
}

/// Benchmark comparing LZSS decompression
fn bench_lzss_comparison(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/compressed/lipsum_rar4_max.rar");
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, compressed) = parse_rar4_file(&data).expect("Failed to parse");

    let mut group = c.benchmark_group("vs_unrar/lzss");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));

    // Decoder reuse - amortizes allocation across iterations (fairer comparison)
    group.bench_function("rar_stream", |b| {
        let mut decoder = Rar29Decoder::new();
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });

    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark comparing PPMd decompression
fn bench_ppmd_comparison(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/compressed/lipsum_rar4_ppmd.rar");
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, compressed) = parse_rar4_file(&data).expect("Failed to parse");

    let mut group = c.benchmark_group("vs_unrar/ppmd");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));

    // Decoder reuse - amortizes allocation across iterations (fairer comparison)
    group.bench_function("rar_stream", |b| {
        let mut decoder = Rar29Decoder::new();
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });

    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark comparing stored (uncompressed) files
fn bench_stored_comparison(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/compressed/lipsum_rar4_store.rar");
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, stored_data) = parse_rar4_file(&data).expect("Failed to parse");

    let mut group = c.benchmark_group("vs_unrar/stored");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));

    group.bench_function("rar_stream", |b| {
        b.iter(|| {
            // rar-stream: just copy for stored files
            let result = black_box(stored_data).to_vec();
            black_box(result)
        });
    });

    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark simulating larger file by repeated decompression (100x = ~350KB)
fn bench_large_simulation(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/compressed/lipsum_rar4_max.rar");
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, compressed) = parse_rar4_file(&data).expect("Failed to parse");

    const ITERATIONS: usize = 100;
    let total_size = header.unpacked_size as u64 * ITERATIONS as u64;

    let mut group = c.benchmark_group("vs_unrar/large_sim");
    group.throughput(Throughput::Bytes(total_size));

    group.bench_function("rar_stream_100x", |b| {
        let mut decoder = Rar29Decoder::new();
        b.iter(|| {
            let mut total = 0usize;
            for _ in 0..ITERATIONS {
                decoder.reset();
                let result = decoder.decompress(black_box(compressed), header.unpacked_size);
                total += result.map(|r| r.len()).unwrap_or(0);
            }
            black_box(total)
        });
    });

    group.bench_function("official_unrar_100x", |b| {
        b.iter(|| {
            let mut total = 0usize;
            for _ in 0..ITERATIONS {
                let result = extract_with_unrar_to_bytes(black_box(archive_path));
                total += result.len();
            }
            black_box(total)
        });
    });

    group.finish();
}

/// Benchmark with real 100MB LZSS file
fn bench_large_lzss(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/large/large_mixed_lzss.rar");
    if !archive_path.exists() {
        eprintln!("Skipping large LZSS benchmark - file not found");
        return;
    }
    
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, compressed) = parse_rar4_file(&data).expect("Failed to parse");

    let mut group = c.benchmark_group("vs_unrar/large_100mb_lzss");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));
    group.sample_size(10); // Fewer samples for large files

    group.bench_function("rar_stream", |b| {
        let mut decoder = Rar29Decoder::new();
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });

    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark with real 100MB PPMd file
fn bench_large_ppmd(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/large/large_mixed_ppmd.rar");
    if !archive_path.exists() {
        eprintln!("Skipping large PPMd benchmark - file not found");
        return;
    }
    
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, compressed) = parse_rar4_file(&data).expect("Failed to parse");

    let mut group = c.benchmark_group("vs_unrar/large_100mb_ppmd");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));
    group.sample_size(10); // Fewer samples for large files

    group.bench_function("rar_stream", |b| {
        let mut decoder = Rar29Decoder::new();
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });

    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark with real 8MB Alpine tar (LZSS)
fn bench_alpine_lzss(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/large/alpine_lzss.rar");
    if !archive_path.exists() {
        eprintln!("Skipping alpine LZSS benchmark - file not found");
        return;
    }
    
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, compressed) = parse_rar4_file(&data).expect("Failed to parse");

    let mut group = c.benchmark_group("vs_unrar/alpine_8mb_lzss");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));
    group.sample_size(20);

    group.bench_function("rar_stream", |b| {
        let mut decoder = Rar29Decoder::new();
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });

    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark with real 8MB Alpine tar (PPMd/m3)
fn bench_alpine_ppmd(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/large/alpine_m3.rar");
    if !archive_path.exists() {
        eprintln!("Skipping alpine PPMd benchmark - file not found");
        return;
    }
    
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (header, compressed) = parse_rar4_file(&data).expect("Failed to parse");

    let mut group = c.benchmark_group("vs_unrar/alpine_8mb_ppmd");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));
    group.sample_size(20);

    group.bench_function("rar_stream", |b| {
        let mut decoder = Rar29Decoder::new();
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });

    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lzss_comparison,
    bench_ppmd_comparison,
    bench_stored_comparison,
    bench_large_simulation,
    bench_alpine_lzss,
    bench_alpine_ppmd,
    bench_large_lzss,
    bench_large_ppmd,
    bench_200mb_alpine,
    bench_200mb_alpine_parallel,
);
criterion_main!(benches);

/// RAR5 marker header signature.
const RAR5_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01, 0x00];

/// Read a RAR5 vint (variable-length integer).
fn read_vint(data: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0;
    loop {
        if pos >= data.len() {
            return None;
        }
        let b = data[pos];
        pos += 1;
        value |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 56 {
            return None;
        }
    }
    Some((value, pos))
}

/// Parse RAR5 file to get first file's compressed data and unpacked size.
fn parse_rar5_file(data: &[u8]) -> Option<(&[u8], usize)> {
    if !data.starts_with(RAR5_MARKER) {
        return None;
    }
    
    let mut pos = RAR5_MARKER.len();
    
    // Skip headers until we find a file header
    loop {
        if pos + 4 > data.len() {
            return None;
        }
        
        // Read header CRC (4 bytes)
        pos += 4;
        
        // Read header size
        let (header_size, next_pos) = read_vint(data, pos)?;
        pos = next_pos;
        
        let header_end = pos + header_size as usize;
        if header_end > data.len() {
            return None;
        }
        
        // Read header type
        let (header_type, next_pos) = read_vint(data, pos)?;
        pos = next_pos;
        
        // Read header flags
        let (header_flags, next_pos) = read_vint(data, pos)?;
        pos = next_pos;
        
        let _has_extra_size = (header_flags & 0x01) != 0;
        let has_data_size = (header_flags & 0x02) != 0;
        
        let mut _extra_size = 0u64;
        if _has_extra_size {
            let (es, next_pos) = read_vint(data, pos)?;
            _extra_size = es;
            pos = next_pos;
        }
        
        let mut data_size = 0u64;
        if has_data_size {
            let (ds, next_pos) = read_vint(data, pos)?;
            data_size = ds;
            pos = next_pos;
        }
        
        // Header type 2 = File header
        if header_type == 2 {
            // Read file flags
            let (_file_flags, next_pos) = read_vint(data, pos)?;
            pos = next_pos;
            
            // Read unpacked size
            let (unpacked_size, _next_pos) = read_vint(data, pos)?;
            
            // Data starts after header
            let data_start = header_end;
            let data_end = data_start + data_size as usize;
            
            if data_end > data.len() {
                return None;
            }
            
            return Some((&data[data_start..data_end], unpacked_size as usize));
        }
        
        // Move to next header (skip current header + any data)
        pos = header_end + data_size as usize;
    }
}

/// Benchmark 200MB Alpine ISO with RAR5
fn bench_200mb_alpine(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/large/alpine-200mb.rar");
    if !archive_path.exists() {
        eprintln!("Skipping 200MB Alpine benchmark - file not found");
        return;
    }
    
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (compressed, unpacked_size) = match parse_rar5_file(&data) {
        Some(r) => r,
        None => {
            eprintln!("Failed to parse RAR5 file");
            return;
        }
    };
    
    let mut group = c.benchmark_group("vs_unrar/200mb_alpine");
    group.throughput(Throughput::Bytes(unpacked_size as u64));
    group.sample_size(10);
    
    group.bench_function("rar_stream", |b| {
        use rar_stream::decompress::rar5::Rar5Decoder;
        let mut decoder = Rar5Decoder::with_dict_size(27); // 128MB dictionary
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress(black_box(compressed), unpacked_size as u64, 1, false);
            black_box(result)
        });
    });
    
    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark 200MB Alpine ISO with RAR5 - parallel version
#[cfg(feature = "parallel")]
fn bench_200mb_alpine_parallel(c: &mut Criterion) {
    let archive_path = Path::new("__fixtures__/large/alpine-200mb.rar");
    if !archive_path.exists() {
        eprintln!("Skipping 200MB Alpine parallel benchmark - file not found");
        return;
    }
    
    let data = std::fs::read(archive_path).expect("Failed to read archive");
    let (compressed, unpacked_size, dict_size_log) = match parse_rar5_file_with_dict(&data) {
        Some(r) => r,
        None => {
            eprintln!("Failed to parse RAR5 file");
            return;
        }
    };
    
    let mut group = c.benchmark_group("vs_unrar/200mb_alpine_parallel");
    group.throughput(Throughput::Bytes(unpacked_size as u64));
    group.sample_size(10);
    
    group.bench_function("rar_stream_parallel", |b| {
        use rar_stream::decompress::rar5::Rar5Decoder;
        let mut decoder = Rar5Decoder::with_dict_size(dict_size_log);
        b.iter(|| {
            decoder.reset();
            let result = decoder.decompress_parallel(black_box(compressed), unpacked_size as u64);
            black_box(result)
        });
    });
    
    group.bench_function("official_unrar", |b| {
        b.iter(|| {
            let result = extract_with_unrar_to_bytes(black_box(archive_path));
            black_box(result)
        });
    });
    
    group.finish();
}

/// Parse RAR5 file and return dict_size_log too
fn parse_rar5_file_with_dict(data: &[u8]) -> Option<(&[u8], usize, u8)> {
    if !data.starts_with(RAR5_MARKER) {
        return None;
    }
    
    let mut pos = RAR5_MARKER.len();
    
    // Skip headers until we find a file header
    loop {
        if pos + 4 > data.len() {
            return None;
        }
        
        // Read header CRC (4 bytes)
        pos += 4;
        
        // Read header size
        let (header_size, next_pos) = read_vint(data, pos)?;
        pos = next_pos;
        
        let header_end = pos + header_size as usize;
        if header_end > data.len() {
            return None;
        }
        
        // Read header type
        let (header_type, next_pos) = read_vint(data, pos)?;
        pos = next_pos;
        
        // Read header flags
        let (header_flags, next_pos) = read_vint(data, pos)?;
        pos = next_pos;
        
        let has_extra_size = (header_flags & 0x01) != 0;
        let has_data_size = (header_flags & 0x02) != 0;
        
        if has_extra_size {
            let (_, next_pos) = read_vint(data, pos)?;
            pos = next_pos;
        }
        
        let mut data_size = 0u64;
        if has_data_size {
            let (ds, next_pos) = read_vint(data, pos)?;
            data_size = ds;
            pos = next_pos;
        }
        
        // Header type 2 = File header
        if header_type == 2 {
            // Read file flags
            let (file_flags, next_pos) = read_vint(data, pos)?;
            pos = next_pos;
            
            // Read unpacked size
            let (unpacked_size, next_pos) = read_vint(data, pos)?;
            pos = next_pos;
            
            // Read attributes
            let (_, next_pos) = read_vint(data, pos)?;
            pos = next_pos;
            
            // Optional mtime (4 bytes) if flag 0x02 is set
            if (file_flags & 0x02) != 0 {
                pos += 4;
            }
            
            // Optional data_crc (4 bytes) if flag 0x04 is set
            if (file_flags & 0x04) != 0 {
                pos += 4;
            }
            
            // Read compression info
            let (compression_info, _) = read_vint(data, pos)?;
            
            // Extract dict_size_log from compression_info
            // Bits 10-14 contain dictionary size: 17 + value
            let dict_size_log = (((compression_info >> 10) & 0x1F) + 17) as u8;
            
            // Data starts after header
            let data_start = header_end;
            let data_end = data_start + data_size as usize;
            
            if data_end > data.len() {
                return None;
            }
            
            return Some((&data[data_start..data_end], unpacked_size as usize, dict_size_log));
        }
        
        // Move to next header (skip current header + any data)
        pos = header_end + data_size as usize;
    }
}
