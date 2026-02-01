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
);
criterion_main!(benches);
