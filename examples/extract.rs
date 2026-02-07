//! Parse a RAR archive, list inner files, and extract them to disk.
//!
//! Usage:
//!   cargo run --release --example extract --features async -- archive.rar output_dir/

use rar_stream::{FileMedia, LocalFileMedia, ParseOptions, RarFilesPackage};
use std::path::Path;
use std::sync::Arc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: extract <archive.rar> <output_dir>");
        eprintln!("  extract ./movie.rar ./out/");
        std::process::exit(1);
    }

    let archive_path = &args[1];
    let output_dir = Path::new(&args[2]);

    std::fs::create_dir_all(output_dir)?;

    let file: Arc<dyn FileMedia> = Arc::new(LocalFileMedia::new(archive_path)?);
    let package = RarFilesPackage::new(vec![file]);
    let files = package.parse(ParseOptions::default()).await?;

    println!("{} file(s) in archive:", files.len());
    for f in &files {
        println!("  {} ({:.2} MB)", f.name, f.length as f64 / 1024.0 / 1024.0);
    }

    for f in &files {
        let content = f.read_to_end().await?;
        let out_path = output_dir.join(&f.name);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out_path, &content)?;
        println!("Extracted {} ({} bytes)", f.name, content.len());
    }

    Ok(())
}
