use std::path::Path;

const RAR5_MARKER: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01, 0x00];

fn read_vint(data: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0;
    loop {
        if pos >= data.len() {
            return None;
        }
        let b = data[pos];
        pos += 1;
        value |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    Some((value, pos))
}

fn parse(data: &[u8]) -> Option<(&[u8], usize, u8)> {
    if !data.starts_with(RAR5_MARKER) {
        return None;
    }
    let mut pos = RAR5_MARKER.len();
    loop {
        if pos + 4 > data.len() {
            return None;
        }
        pos += 4;
        let (hs, np) = read_vint(data, pos)?;
        pos = np;
        let he = pos + hs as usize;
        if he > data.len() {
            return None;
        }
        let (ht, np) = read_vint(data, pos)?;
        pos = np;
        let (hf, np) = read_vint(data, pos)?;
        pos = np;
        if (hf & 0x01) != 0 {
            let (_, np) = read_vint(data, pos)?;
            pos = np;
        }
        let mut ds = 0u64;
        if (hf & 0x02) != 0 {
            let (d, np) = read_vint(data, pos)?;
            ds = d;
            pos = np;
        }
        if ht == 2 {
            let (ff, np) = read_vint(data, pos)?;
            pos = np;
            let (us, np) = read_vint(data, pos)?;
            pos = np;
            let (_, np) = read_vint(data, pos)?;
            pos = np;
            if (ff & 0x02) != 0 {
                pos += 4;
            }
            if (ff & 0x04) != 0 {
                pos += 4;
            }
            let (ci, _) = read_vint(data, pos)?;
            let dsl = (((ci >> 10) & 0x1F) + 17) as u8;
            let dstart = he;
            let dend = dstart + ds as usize;
            if dend > data.len() {
                return None;
            }
            return Some((&data[dstart..dend], us as usize, dsl));
        }
        pos = he + ds as usize;
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: profile_decode <archive.rar> [method] [--verify]");
    let data = std::fs::read(&path).expect("read");
    let (compressed, unpacked_size, dict_size_log) = parse(&data).expect("parse");

    let method = std::env::args().nth(2).unwrap_or("pipeline".to_string());
    let verify = std::env::args().any(|a| a == "--verify");

    use rar_stream::decompress::rar5::Rar5Decoder;

    if verify {
        let mut results: Vec<(String, Vec<u8>)> = Vec::new();
        for m in &["single", "pipeline"] {
            let mut decoder = Rar5Decoder::with_dict_size(dict_size_log);
            decoder.reset();
            let output = match *m {
                "single" => decoder
                    .decompress(compressed, unpacked_size as u64, 1, false)
                    .unwrap(),
                "pipeline" => decoder
                    .decompress_pipeline(compressed, unpacked_size as u64)
                    .unwrap(),
                _ => unreachable!(),
            };
            eprintln!("{}: {} bytes", m, output.len());
            results.push((m.to_string(), output));
        }
        let first = &results[0].1;
        for (name, output) in &results[1..] {
            if output == first {
                eprintln!("{} matches single ✓", name);
            } else {
                let diff_pos = first
                    .iter()
                    .zip(output.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(first.len().min(output.len()));
                eprintln!("{} DIFFERS from single ✗ (byte {})", name, diff_pos);
                std::process::exit(1);
            }
        }
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("md5sum")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(first).unwrap();
        let out = child.wait_with_output().unwrap();
        eprintln!("md5: {}", String::from_utf8_lossy(&out.stdout).trim());
    } else {
        let mut decoder = Rar5Decoder::with_dict_size(dict_size_log);
        decoder.reset();
        match method.as_str() {
            "single" => {
                let _ = decoder.decompress(compressed, unpacked_size as u64, 1, false);
            }
            "pipeline" => {
                let _ = decoder.decompress_pipeline(compressed, unpacked_size as u64);
            }
            _ => panic!("unknown method"),
        }

        decoder.reset();
        let start = std::time::Instant::now();
        match method.as_str() {
            "single" => {
                let _ = decoder.decompress(compressed, unpacked_size as u64, 1, false);
            }
            "pipeline" => {
                let _ = decoder.decompress_pipeline(compressed, unpacked_size as u64);
            }
            _ => {}
        }
        let elapsed = start.elapsed();
        eprintln!("{}: {:.1}ms", method, elapsed.as_secs_f64() * 1000.0);
    }
}
