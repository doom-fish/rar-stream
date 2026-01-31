//! Benchmarks for RAR decompression performance.
//!
//! Run with: `cargo bench`
//! Compare with baseline: `cargo bench -- --save-baseline main`
//! Compare against baseline: `cargo bench -- --baseline main`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rar_stream::Rar29Decoder;
use rar_stream::parsing::file_header::FileHeaderParser;

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

/// Benchmark RAR4 LZSS decompression (default compression)
fn bench_lzss_default(c: &mut Criterion) {
    let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_default.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
    
    let mut group = c.benchmark_group("decompress");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));
    
    group.bench_function("lzss_default", |b| {
        b.iter(|| {
            let mut decoder = Rar29Decoder::new();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark RAR4 LZSS decompression (max compression)
fn bench_lzss_max(c: &mut Criterion) {
    let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_max.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
    
    let mut group = c.benchmark_group("decompress");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));
    
    group.bench_function("lzss_max", |b| {
        b.iter(|| {
            let mut decoder = Rar29Decoder::new();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark RAR4 PPMd decompression
fn bench_ppmd(c: &mut Criterion) {
    let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_ppmd.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
    
    let mut group = c.benchmark_group("decompress");
    group.throughput(Throughput::Bytes(header.unpacked_size as u64));
    
    group.bench_function("ppmd", |b| {
        b.iter(|| {
            let mut decoder = Rar29Decoder::new();
            let result = decoder.decompress(black_box(compressed), header.unpacked_size);
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark RAR header parsing
fn bench_header_parsing(c: &mut Criterion) {
    let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_default.rar");
    
    c.bench_function("parse_header", |b| {
        b.iter(|| {
            // Parse file header from RAR archive
            let result = parse_rar4_file(black_box(data));
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_header_parsing,
    bench_lzss_default,
    bench_lzss_max,
    bench_ppmd,
);
criterion_main!(benches);
