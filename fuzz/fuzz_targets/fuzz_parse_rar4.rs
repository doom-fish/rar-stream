#![no_main]
use libfuzzer_sys::fuzz_target;
use rar_stream::parsing::file_header::FileHeaderParser;
use rar_stream::parsing::marker_header::MarkerHeaderParser;
use rar_stream::parsing::ArchiveHeaderParser;

fuzz_target!(|data: &[u8]| {
    // Fuzz marker header
    let _ = MarkerHeaderParser::parse(data);

    // Fuzz archive header
    let _ = ArchiveHeaderParser::parse(data);

    // Fuzz file header
    let _ = FileHeaderParser::parse(data);
});
