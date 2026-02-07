#![no_main]
use libfuzzer_sys::fuzz_target;
use rar_stream::Rar29Decoder;

fuzz_target!(|data: &[u8]| {
    if data.len() < 9 {
        return;
    }

    // Use first 8 bytes as unpacked_size (capped to 16MB to avoid OOM)
    let unpacked_size = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]) % (16 * 1024 * 1024);

    let compressed = &data[8..];
    let mut decoder = Rar29Decoder::new();
    let _ = decoder.decompress(compressed, unpacked_size);
});
