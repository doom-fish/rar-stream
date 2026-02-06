//! Decompression integration tests.
//!
//! Tests decompression of real RAR archives from __fixtures__/compressed/.

use super::*;
use crate::parsing::file_header::FileHeaderParser;

/// RAR4 marker header signature.
const RAR4_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00];

/// Parse a RAR4 file and extract file header + compressed data.
fn parse_rar4_file(data: &[u8]) -> Option<(crate::parsing::file_header::FileHeader, &[u8])> {
    // Check marker
    if !data.starts_with(RAR4_MARKER) {
        return None;
    }

    let mut pos = RAR4_MARKER.len();

    // Skip archive header
    if data.len() < pos + 7 {
        return None;
    }
    let archive_head_size = u16::from_le_bytes([data[pos + 5], data[pos + 6]]) as usize;
    pos += archive_head_size;

    // Parse file header
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

#[test]
fn test_parse_stored_rar() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_store.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "lorem_ipsum.txt");
    assert_eq!(header.method, 0x30); // Stored
    assert_eq!(header.unpacked_size, 3515);
    assert_eq!(header.packed_size, 3515); // Same size for stored

    // For stored files, data should be identical
    let expected = include_bytes!("../../__fixtures__/compressed/lorem_ipsum.txt.expected");
    assert_eq!(compressed, expected.as_slice());
}

#[test]
fn test_decompress_lzss_max() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_max.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "lorem_ipsum.txt");
    assert!(header.method >= 0x31 && header.method <= 0x35); // Compressed
    assert_eq!(header.unpacked_size, 3515);

    let mut decoder = Rar29Decoder::new();
    let decompressed = decoder
        .decompress(compressed, header.unpacked_size)
        .expect("LZSS decompression failed");

    let expected = include_bytes!("../../__fixtures__/compressed/lorem_ipsum.txt.expected");
    assert_eq!(decompressed.len(), expected.len(), "Size mismatch");
    assert_eq!(
        decompressed.as_slice(),
        expected.as_slice(),
        "Content mismatch"
    );
}

#[test]
fn test_decompress_lzss_default() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_default.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "lorem_ipsum.txt");
    assert_eq!(header.unpacked_size, 3515);

    let mut decoder = Rar29Decoder::new();
    let decompressed = decoder
        .decompress(compressed, header.unpacked_size)
        .expect("LZSS default decompression failed");

    let expected = include_bytes!("../../__fixtures__/compressed/lorem_ipsum.txt.expected");
    assert_eq!(decompressed.as_slice(), expected.as_slice());
}

#[test]
fn test_decompress_ppmd() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_ppmd.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "lorem_ipsum.txt");
    assert_eq!(header.unpacked_size, 3515);

    let mut decoder = Rar29Decoder::new();
    let decompressed = decoder
        .decompress(compressed, header.unpacked_size)
        .expect("PPMd decompression failed");

    let expected = include_bytes!("../../__fixtures__/compressed/lorem_ipsum.txt.expected");
    assert_eq!(decompressed.len(), expected.len(), "Size mismatch");
    assert_eq!(
        decompressed.as_slice(),
        expected.as_slice(),
        "Content mismatch"
    );
}

#[test]
fn test_decompress_delta() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_delta.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "lorem_ipsum.txt");
    assert_eq!(header.unpacked_size, 3515);

    let mut decoder = Rar29Decoder::new();
    let decompressed = decoder
        .decompress(compressed, header.unpacked_size)
        .expect("Delta decompression failed");

    let expected = include_bytes!("../../__fixtures__/compressed/lorem_ipsum.txt.expected");
    assert_eq!(decompressed.as_slice(), expected.as_slice());
}

#[test]
#[ignore = "Audio filter requires multi-block solid archive support"]
fn test_decompress_audio() {
    let data = include_bytes!("../../__fixtures__/compressed/silent_rar4_audio.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "silent_quarter-second.wav");
    assert_eq!(header.unpacked_size, 44292);

    let mut decoder = Rar29Decoder::new();
    let decompressed = decoder
        .decompress(compressed, header.unpacked_size)
        .expect("Audio decompression failed");

    // Check size matches
    assert_eq!(decompressed.len(), 44292, "Size mismatch");

    // Check WAV header (RIFF....WAVEfmt )
    assert_eq!(&decompressed[0..4], b"RIFF", "Invalid WAV header");
    assert_eq!(&decompressed[8..12], b"WAVE", "Invalid WAV format");
}

// Legacy tests for compatibility
#[test]
fn test_parse_lzss_rar() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_max.rar");
    let (header, _compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "lorem_ipsum.txt");
    assert!(header.method >= 0x31 && header.method <= 0x35);
    assert_eq!(header.unpacked_size, 3515);
    assert!(header.packed_size < header.unpacked_size);
}

#[test]
fn test_parse_ppmd_rar() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_ppmd.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    assert_eq!(header.name, "lorem_ipsum.txt");
    assert_eq!(header.unpacked_size, 3515);

    let mut decoder = Rar29Decoder::new();
    let decompressed = decoder
        .decompress(compressed, header.unpacked_size)
        .expect("PPMd decompression failed");

    let expected = include_bytes!("../../__fixtures__/compressed/lorem_ipsum.txt.expected");
    assert_eq!(decompressed.as_slice(), expected.as_slice());
    println!(
        "PPMd decompression: SUCCESS - {} bytes match perfectly!",
        decompressed.len()
    );
}

#[test]
fn test_lzss_decompression() {
    let data = include_bytes!("../../__fixtures__/compressed/lipsum_rar4_max.rar");
    let (header, compressed) = parse_rar4_file(data).expect("Failed to parse RAR file");

    let mut decoder = Rar29Decoder::new();
    let decompressed = decoder
        .decompress(compressed, header.unpacked_size)
        .expect("LZSS decompression failed");

    let expected = include_bytes!("../../__fixtures__/compressed/lorem_ipsum.txt.expected");
    assert_eq!(decompressed.as_slice(), expected.as_slice());
}

/// Test decompression of a larger LZSS file (alpine.tar, ~8MB).
/// This tests the output buffer accumulation and length bonus for long distances.
/// TODO: There's a remaining issue at byte 46592 that needs investigation.
#[test]
fn test_decompress_large_lzss() {
    use std::io::Read;

    let mut file = std::fs::File::open("__fixtures__/large/alpine_lzss.rar").unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    // Parse to find compressed data
    let file_header_pos = 20; // After marker + archive header
    let header_size =
        u16::from_le_bytes([data[file_header_pos + 5], data[file_header_pos + 6]]) as usize;
    let pack_size = u32::from_le_bytes([
        data[file_header_pos + 7],
        data[file_header_pos + 8],
        data[file_header_pos + 9],
        data[file_header_pos + 10],
    ]) as usize;
    let unp_size = u32::from_le_bytes([
        data[file_header_pos + 11],
        data[file_header_pos + 12],
        data[file_header_pos + 13],
        data[file_header_pos + 14],
    ]) as u64;

    let comp_start = file_header_pos + header_size;
    let compressed = &data[comp_start..comp_start + pack_size];

    eprintln!(
        "TEST: comp_start={}, pack_size={}, unp_size={}",
        comp_start, pack_size, unp_size
    );
    eprintln!(
        "TEST: first 8 bytes of compressed: {:02x?}",
        &compressed[0..8]
    );
    eprintln!("TEST: bytes at 38964: {:02x?}", &compressed[38964..38972]);

    let mut decoder = super::rar29::Rar29Decoder::new();
    let result = match decoder.decompress(compressed, unp_size) {
        Ok(r) => r,
        Err(e) => {
            // Check partial output for mismatches
            let partial = decoder.get_output();
            let mut orig_file = std::fs::File::open("__fixtures__/large/alpine.tar").unwrap();
            let mut orig_data = Vec::new();
            orig_file.read_to_end(&mut orig_data).unwrap();

            for (i, (a, b)) in partial.iter().zip(orig_data.iter()).enumerate() {
                if *a != *b {
                    panic!("Decompression failed with {:?}. First mismatch at byte {}: got 0x{:02x}, expected 0x{:02x}", e, i, a, b);
                }
            }
            panic!(
                "Decompression failed with {:?} at output byte {}, but all bytes match original",
                e,
                partial.len()
            );
        }
    };

    assert_eq!(
        result.len(),
        unp_size as usize,
        "Decompressed size mismatch"
    );

    // Verify against original file
    let mut orig_file = std::fs::File::open("__fixtures__/large/alpine.tar").unwrap();
    let mut orig_data = Vec::new();
    orig_file.read_to_end(&mut orig_data).unwrap();

    // Find first mismatch if any
    for (i, (a, b)) in result.iter().zip(orig_data.iter()).enumerate() {
        if a != b {
            panic!(
                "Mismatch at byte {}: got 0x{:02x}, expected 0x{:02x}",
                i, a, b
            );
        }
    }
}
