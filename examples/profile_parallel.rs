use std::path::Path;

fn read_vint(data: &[u8], pos: &mut usize) -> u64 {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        let byte = data[*pos];
        *pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 { break; }
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
    
    #[cfg(feature = "parallel")]
    {
        use rar_stream::decompress::rar5::block_decoder::{Rar5BlockDecoder, ParallelConfig};
        
        // Use parallel decoder directly
        let mut block_decoder = Rar5BlockDecoder::new(file_header.compression.dict_size_log);
        let config = ParallelConfig::default();
        
        // First test single-threaded decode
        eprintln!("Testing single-threaded decode first...");
        let mut single_decoder = Rar5BlockDecoder::new(file_header.compression.dict_size_log);
        use rar_stream::decompress::rar5::bit_decoder::BitDecoder;
        let mut bits = BitDecoder::new(compressed);
        
        let _ = single_decoder.decode_block(&mut bits, 100).unwrap(); // Decode first 100 bytes
        let single_output = single_decoder.get_output(0, 100);
        eprintln!("Single-threaded first 16 bytes: {:02x?}", &single_output[..16.min(single_output.len())]);

        eprintln!("Using {} threads", rayon::current_num_threads());
        eprintln!("dict_size_log: {}, dict_size: {} MB", 
                  file_header.compression.dict_size_log, 
                  1 << file_header.compression.dict_size_log >> 20);
        eprintln!("packed_size: {}, unpacked_size: {}", 
                  file_header.packed_size, file_header.unpacked_size);
        let result = block_decoder.decode_parallel_with_config(
            compressed,
            file_header.unpacked_size as usize,
            &config
        );
        
        match result {
            Ok(output) => {
                eprintln!("Decompressed {} bytes", output.len());
                eprintln!("First 32 bytes: {:02x?}", &output[..32.min(output.len())]);
                
                // Find where 33 ed 90 90 appears
                for i in 0..output.len().min(1000) {
                    if output[i] == 0x33 && output.get(i+1) == Some(&0xed) && output.get(i+2) == Some(&0x90) {
                        eprintln!("Found 33 ed 90 at offset {}", i);
                        break;
                    }
                }
                
                // Verify hash
                let expected = std::fs::read("__fixtures__/large/alpine-200mb.iso").ok();
                if let Some(expected) = expected {
                    if output == expected {
                        eprintln!("Hash verified: OK");
                    } else {
                        // Find first difference
                        for (i, (a, b)) in output.iter().zip(expected.iter()).enumerate() {
                            if a != b {
                                eprintln!("MISMATCH at byte {}: got {:02x}, expected {:02x}", i, a, b);
                                break;
                            }
                        }
                        if output.len() != expected.len() {
                            eprintln!("Size mismatch: got {}, expected {}", output.len(), expected.len());
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
    
    #[cfg(not(feature = "parallel"))]
    {
        eprintln!("Parallel feature not enabled");
    }
}
