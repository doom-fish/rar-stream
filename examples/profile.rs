//! Profiling helper - runs decompression in a loop for perf analysis
//!
//! Usage:
//!   cargo run --release --example profile -- lzss 10000       # small file
//!   cargo run --release --example profile -- alpine-lzss 100  # 8MB file
//!   cargo run --release --example profile -- alpine-ppmd 100  # 8MB PPMd

use rar_stream::parsing::file_header::FileHeaderParser;
use rar_stream::Rar29Decoder;

const RAR4_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00];

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

fn run_decompress(mode: &str, iterations: usize) {
    match mode {
        "lzss" => {
            let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_max.rar");
            let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
            run_iterations("LZSS (3.5KB)", compressed, header.unpacked_size, iterations);
        }
        "ppmd" => {
            let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_ppmd.rar");
            let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
            run_iterations("PPMd (3.5KB)", compressed, header.unpacked_size, iterations);
        }
        "alpine-lzss" => {
            let data = include_bytes!("../__fixtures__/large/alpine_lzss.rar");
            let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
            run_iterations("Alpine LZSS (8MB)", compressed, header.unpacked_size, iterations);
        }
        "alpine-ppmd" => {
            let data = include_bytes!("../__fixtures__/large/alpine_m3.rar");
            let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
            run_iterations("Alpine PPMd (8MB)", compressed, header.unpacked_size, iterations);
        }
        _ => {
            eprintln!("Usage: profile [lzss|ppmd|alpine-lzss|alpine-ppmd] [iterations]");
            eprintln!("  lzss         - 3.5KB lorem ipsum LZSS");
            eprintln!("  ppmd         - 3.5KB lorem ipsum PPMd");
            eprintln!("  alpine-lzss  - 8MB Alpine tar LZSS");
            eprintln!("  alpine-ppmd  - 8MB Alpine tar PPMd");
        }
    }
}

fn run_iterations(name: &str, compressed: &[u8], unpacked_size: u64, iterations: usize) {
    println!("Running {} decompression {} times...", name, iterations);
    
    let start = std::time::Instant::now();
    
    let mut decoder = Rar29Decoder::new();
    for _ in 0..iterations {
        decoder.reset();
        let _ = decoder.decompress(compressed, unpacked_size);
    }
    
    let elapsed = start.elapsed();
    let total_bytes = unpacked_size as usize * iterations;
    let throughput = total_bytes as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0;
    println!("Done: {:?} total, {:.1} MiB/s", elapsed, throughput);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("lzss");
    let iterations: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);

    run_decompress(mode, iterations);
}
