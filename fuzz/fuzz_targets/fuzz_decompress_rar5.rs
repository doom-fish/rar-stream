#![no_main]
use libfuzzer_sys::fuzz_target;
use rar_stream::decompress::rar5::Rar5Decoder;

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }

    // First 8 bytes: unpacked_size (capped to 16MB)
    let unpacked_size = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]) % (16 * 1024 * 1024);

    // Byte 8: dict_size_log (17-28 valid range)
    let dict_size_log = 17 + (data[8] % 12);

    // Byte 9: method (0-5)
    let method = data[9] % 6;

    let compressed = &data[10..];
    let mut decoder = Rar5Decoder::with_dict_size(dict_size_log);
    let _ = decoder.decompress(compressed, unpacked_size, method, false);
});
