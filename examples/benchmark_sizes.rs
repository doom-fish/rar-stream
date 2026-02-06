//! Benchmark decompression across various file sizes.
//!
//! Run with: cargo run --release --example benchmark_sizes

use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

use rar_stream::decompress::rar5::Rar5Decoder;
use rar_stream::decompress::Rar29Decoder;
use rar_stream::parsing::file_header::FileHeaderParser;
use rar_stream::parsing::marker_header::{MarkerHeaderParser, RarVersion};
use rar_stream::parsing::rar5::Rar5FileHeaderParser;

const RAR4_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00];
const RAR5_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01, 0x00];

fn parse_and_decompress_rar4(data: &[u8]) -> Option<Vec<u8>> {
    if !data.starts_with(RAR4_MARKER) {
        return None;
    }

    let mut pos = RAR4_MARKER.len();
    let archive_head_size = u16::from_le_bytes([data[pos + 5], data[pos + 6]]) as usize;
    pos += archive_head_size;

    let header = FileHeaderParser::parse(&data[pos..]).ok()?;
    let data_start = pos + header.head_size as usize;
    let data_end = data_start + header.packed_size as usize;
    let compressed = &data[data_start..data_end];

    let mut decoder = Rar29Decoder::new();
    decoder.decompress(compressed, header.unpacked_size).ok()
}

fn parse_and_decompress_rar5(data: &[u8]) -> Option<Vec<u8>> {
    if !data.starts_with(RAR5_MARKER) {
        return None;
    }

    let mut pos = RAR5_MARKER.len();

    // Skip archive header
    let _crc = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
    pos += 4;

    // Read header size vint
    let (header_size, vint_len) = read_vint(&data[pos..])?;
    pos += vint_len + header_size as usize;

    // Parse file header
    let (file_header, _) = Rar5FileHeaderParser::parse(&data[pos..]).ok()?;
    let data_start = pos + file_header.header_size as usize;
    let data_end = data_start + file_header.packed_size as usize;
    let compressed = &data[data_start..data_end];

    let mut decoder = Rar5Decoder::with_dict_size(file_header.compression.dict_size_log);
    decoder
        .decompress(
            compressed,
            file_header.unpacked_size,
            file_header.compression.method,
            file_header.compression.is_solid,
        )
        .ok()
}

fn read_vint(data: &[u8]) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in data.iter().enumerate() {
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
    None
}

fn benchmark_file(path: &str) -> Option<(usize, f64)> {
    let mut file = fs::File::open(path).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;

    // Detect version
    let is_rar5 = data.starts_with(RAR5_MARKER);

    // Warmup
    let output = if is_rar5 {
        parse_and_decompress_rar5(&data)?
    } else {
        parse_and_decompress_rar4(&data)?
    };
    let output_size = output.len();

    // Benchmark
    let runs = if output_size > 50 * 1024 * 1024 {
        3
    } else if output_size > 1024 * 1024 {
        5
    } else {
        20
    };

    let start = Instant::now();
    for _ in 0..runs {
        if is_rar5 {
            let _ = parse_and_decompress_rar5(&data);
        } else {
            let _ = parse_and_decompress_rar4(&data);
        }
    }
    let elapsed = start.elapsed().as_secs_f64() / runs as f64;

    Some((output_size, elapsed))
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

fn format_time(secs: f64) -> String {
    if secs < 0.001 {
        format!("{:.0} µs", secs * 1_000_000.0)
    } else if secs < 1.0 {
        format!("{:.1} ms", secs * 1000.0)
    } else {
        format!("{:.2} s", secs)
    }
}

fn main() {
    println!("=== PURE RUST DECOMPRESSION BENCHMARKS ===\n");
    println!(
        "{:<35} {:>12} {:>12} {:>10}",
        "File", "Size", "Time", "Speed"
    );
    println!("{}", "-".repeat(75));

    let test_files: Vec<(&str, &str)> = vec![
        // Small fixture files
        (
            "lorem LZSS (RAR4)",
            "__fixtures__/compressed/lipsum_rar4_default.rar",
        ),
        (
            "lorem PPMd (RAR4)",
            "__fixtures__/compressed/lipsum_rar4_ppmd.rar",
        ),
        (
            "lorem Store (RAR4)",
            "__fixtures__/compressed/lipsum_rar4_store.rar",
        ),
        // Size boundary tests
        ("1 byte", "__fixtures__/sizes/test_1_rar5.rar"),
        ("256 bytes", "__fixtures__/sizes/test_256_rar5.rar"),
        ("1 KB", "__fixtures__/sizes/test_1024_rar5.rar"),
        ("4 KB", "__fixtures__/sizes/test_4096_rar5.rar"),
        ("32 KB", "__fixtures__/sizes/test_32768_rar5.rar"),
        ("64 KB", "__fixtures__/sizes/test_65536_rar5.rar"),
        ("512 KB", "__fixtures__/sizes/test_524288_rar5.rar"),
        ("1 MB", "__fixtures__/sizes/test_1048576_rar5.rar"),
        // Medium files
        (
            "alpine.tar LZSS 8MB (RAR4)",
            "__fixtures__/large/alpine_lzss.rar",
        ),
        (
            "alpine.tar PPMd 8MB (RAR4)",
            "__fixtures__/large/alpine_m3.rar",
        ),
        (
            "alpine.tar 8MB (RAR5)",
            "__fixtures__/large/alpine_rar5.rar",
        ),
        // Large benchmark files
        ("ISO 10MB", "/tmp/rar-benchmark/test_10mb_rar5.rar"),
        ("ISO 50MB", "/tmp/rar-benchmark/test_50mb_rar5.rar"),
        ("ISO 100MB", "/tmp/rar-benchmark/test_100mb_rar5.rar"),
        ("ISO 300MB", "/tmp/rar-benchmark/test_300mb_rar5.rar"),
        ("ISO 600MB", "/tmp/rar-benchmark/test_600mb_rar5.rar"),
        ("Alpine ISO 1GB (RAR5)", "/tmp/rar-benchmark/rar5-lzss.rar"),
    ];

    let mut results = Vec::new();

    for (name, path) in &test_files {
        if !Path::new(path).exists() {
            continue;
        }

        if let Some((size, time)) = benchmark_file(path) {
            let speed_mbps = (size as f64 / 1024.0 / 1024.0) / time;
            println!(
                "{:<35} {:>12} {:>12} {:>8.0} MB/s",
                name,
                format_size(size),
                format_time(time),
                speed_mbps
            );
            results.push((size, speed_mbps));
        }
    }

    println!("{}", "-".repeat(75));

    // Summary
    let large_results: Vec<_> = results.iter().filter(|(s, _)| *s > 1024 * 1024).collect();
    if !large_results.is_empty() {
        let avg_speed: f64 =
            large_results.iter().map(|(_, s)| s).sum::<f64>() / large_results.len() as f64;
        let max_speed = large_results
            .iter()
            .map(|(_, s)| *s)
            .fold(0.0_f64, f64::max);
        println!("\nSummary (files > 1MB):");
        println!("  Average: {:.0} MB/s", avg_speed);
        println!("  Peak:    {:.0} MB/s", max_speed);
    }
}
