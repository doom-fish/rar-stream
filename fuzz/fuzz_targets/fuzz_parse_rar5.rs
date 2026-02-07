#![no_main]
use libfuzzer_sys::fuzz_target;
use rar_stream::parsing::rar5::{
    Rar5ArchiveHeaderParser, Rar5EncryptionHeaderParser, Rar5EndHeaderParser,
    Rar5FileHeaderParser,
};

fuzz_target!(|data: &[u8]| {
    // Fuzz RAR5 headers
    let _ = Rar5ArchiveHeaderParser::parse(data);
    let _ = Rar5FileHeaderParser::parse(data);
    let _ = Rar5EncryptionHeaderParser::parse(data);
    let _ = Rar5EndHeaderParser::parse(data);
});
