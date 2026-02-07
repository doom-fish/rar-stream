#![no_main]
use libfuzzer_sys::fuzz_target;
use rar_stream::parsing::file_header::FileHeaderParser;
use rar_stream::parsing::marker_header::MarkerHeaderParser;
use rar_stream::parsing::ArchiveHeaderParser;
use rar_stream::Rar29Decoder;

// Fuzz a complete RAR4 archive: parse headers then decompress.
fuzz_target!(|data: &[u8]| {
    // Need at least marker + archive header + some file header
    if data.len() < 50 {
        return;
    }

    let marker = match MarkerHeaderParser::parse(data) {
        Ok(m) => m,
        Err(_) => return,
    };
    let mut offset = marker.size as usize;

    if data.len() < offset + 7 {
        return;
    }
    let archive = match ArchiveHeaderParser::parse(&data[offset..]) {
        Ok(a) => a,
        Err(_) => return,
    };
    offset += archive.size as usize;

    if data.len() < offset + 32 {
        return;
    }
    let header = match FileHeaderParser::parse(&data[offset..]) {
        Ok(h) => h,
        Err(_) => return,
    };

    // Cap unpacked size to prevent OOM and timeouts
    if header.unpacked_size > 1024 * 1024 {
        return;
    }

    let data_start = offset + header.head_size as usize;
    let data_end = data_start + header.packed_size as usize;
    if data_end > data.len() {
        return;
    }

    let compressed = &data[data_start..data_end];
    let mut decoder = Rar29Decoder::new();
    let _ = decoder.decompress(compressed, header.unpacked_size);
});
