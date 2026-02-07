#![no_main]
use libfuzzer_sys::fuzz_target;
use rar_stream::decompress::rar5::Rar5Decoder;
use rar_stream::parsing::rar5::Rar5FileHeaderParser;

// Fuzz a complete RAR5 archive: parse header then decompress.
fuzz_target!(|data: &[u8]| {
    // RAR5 signature (8 bytes) + some header
    const RAR5_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01, 0x00];

    if data.len() < 20 {
        return;
    }

    // Try parsing with and without the RAR5 marker
    let parse_buf = if data.starts_with(RAR5_MARKER) {
        // Skip marker + archive header (simplified: skip first block after marker)
        let mut pos = RAR5_MARKER.len();
        // Skip archive header: CRC(4) + size(vint) + content
        if pos + 4 >= data.len() {
            return;
        }
        pos += 4; // CRC
        // Read size vint
        let mut shift = 0u32;
        let mut header_size = 0u64;
        loop {
            if pos >= data.len() {
                return;
            }
            let b = data[pos];
            pos += 1;
            header_size |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 56 {
                return;
            }
        }
        pos += header_size.min(data.len() as u64) as usize;
        if pos >= data.len() {
            return;
        }
        &data[pos..]
    } else {
        data
    };

    let (header, header_size) = match Rar5FileHeaderParser::parse(parse_buf) {
        Ok(h) => h,
        Err(_) => return,
    };

    // Cap unpacked size to prevent OOM
    if header.unpacked_size > 16 * 1024 * 1024 {
        return;
    }

    let data_start = header_size;
    let data_end = data_start + header.packed_size as usize;
    if data_end > parse_buf.len() {
        return;
    }

    let compressed = &parse_buf[data_start..data_end];
    let mut decoder = Rar5Decoder::with_dict_size(header.compression.dict_size_log);
    let _ = decoder.decompress(
        compressed,
        header.unpacked_size,
        header.compression.method,
        header.compression.is_solid,
    );
});
