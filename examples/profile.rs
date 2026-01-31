//! Profiling helper - runs decompression in a loop for perf analysis

use rar_stream::Rar29Decoder;
use rar_stream::parsing::file_header::FileHeaderParser;

const RAR4_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00];

fn parse_rar4_file(data: &[u8]) -> Option<(rar_stream::parsing::file_header::FileHeader, &[u8])> {
    if !data.starts_with(RAR4_MARKER) { return None; }
    let mut pos = RAR4_MARKER.len();
    if data.len() < pos + 7 { return None; }
    let archive_head_size = u16::from_le_bytes([data[pos + 5], data[pos + 6]]) as usize;
    pos += archive_head_size;
    if data.len() < pos + FileHeaderParser::HEADER_SIZE { return None; }
    let header = FileHeaderParser::parse(&data[pos..]).ok()?;
    let data_start = pos + header.head_size as usize;
    let data_end = data_start + header.packed_size as usize;
    if data.len() < data_end { return None; }
    Some((header, &data[data_start..data_end]))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("lzss");
    let iterations: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10000);
    let reuse = args.iter().any(|s| s == "--reuse" || s == "reuse");
    
    match mode {
        "lzss" => {
            let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_max.rar");
            let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
            
            if reuse {
                println!("Running LZSS decompression {} times (REUSING decoder)...", iterations);
                let mut decoder = Rar29Decoder::new();
                for _ in 0..iterations {
                    decoder.reset();
                    let _ = decoder.decompress(compressed, header.unpacked_size);
                }
            } else {
                println!("Running LZSS decompression {} times (NEW decoder each time)...", iterations);
                for _ in 0..iterations {
                    let mut decoder = Rar29Decoder::new();
                    let _ = decoder.decompress(compressed, header.unpacked_size);
                }
            }
        }
        "ppmd" => {
            let data = include_bytes!("../__fixtures__/compressed/lipsum_rar4_ppmd.rar");
            let (header, compressed) = parse_rar4_file(data).expect("Failed to parse");
            
            if reuse {
                println!("Running PPMd decompression {} times (REUSING decoder)...", iterations);
                let mut decoder = Rar29Decoder::new();
                for _ in 0..iterations {
                    decoder.reset();
                    let _ = decoder.decompress(compressed, header.unpacked_size);
                }
            } else {
                println!("Running PPMd decompression {} times (NEW decoder each time)...", iterations);
                for _ in 0..iterations {
                    let mut decoder = Rar29Decoder::new();
                    let _ = decoder.decompress(compressed, header.unpacked_size);
                }
            }
        }
        _ => eprintln!("Usage: profile [lzss|ppmd] [iterations] [reuse]"),
    }
    println!("Done");
}
