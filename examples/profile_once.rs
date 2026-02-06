use std::path::Path;

fn read_vint(data: &[u8], pos: &mut usize) -> u64 {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        let byte = data[*pos];
        *pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    result
}

fn parse_rar5_header_size(data: &[u8]) -> usize {
    let mut pos = 4;
    let header_size = read_vint(data, &mut pos) as usize;
    4 + (pos - 4) + header_size
}

fn main() {
    let archive_path = Path::new("__fixtures__/large/alpine-200mb.rar");
    let data = std::fs::read(archive_path).expect("Failed to read archive");

    assert!(data.starts_with(&[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01, 0x00]));

    let mut pos = 8;
    let header_size = parse_rar5_header_size(&data[pos..]);
    pos += header_size;

    use rar_stream::parsing::rar5::Rar5FileHeaderParser;
    let (file_header, header_size) = Rar5FileHeaderParser::parse(&data[pos..]).expect("parse");

    let data_start = pos + header_size;
    let data_end = data_start + file_header.packed_size as usize;
    let compressed = &data[data_start..data_end];

    use rar_stream::decompress::Rar5Decoder;
    let mut decoder = Rar5Decoder::with_dict_size(file_header.compression.dict_size_log);
    let result = decoder.decompress(
        compressed,
        file_header.unpacked_size,
        file_header.compression.method,
        false,
    );
    eprintln!("Decompressed {} bytes", result.unwrap().len());
}
