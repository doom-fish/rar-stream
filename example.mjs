#!/usr/bin/env node
// Example: Parse RAR files from a torrent and list inner files
// Usage: node example.mjs <path-to-rar-file-or-directory>

import { RarFilesPackage, LocalFileMedia } from './index.js';
import fs from 'fs';
import path from 'path';

async function main() {
  const input = process.argv[2];
  
  if (!input) {
    console.log('Usage: node example.mjs <rar-file-or-directory>');
    console.log('');
    console.log('Examples:');
    console.log('  node example.mjs ./archive.rar');
    console.log('  node example.mjs ./multi-volume-dir/');
    process.exit(1);
  }

  let rarFiles = [];
  
  const stat = fs.statSync(input);
  if (stat.isDirectory()) {
    // Find all .rar and .rXX files in directory
    const files = fs.readdirSync(input)
      .filter(f => /\.(rar|r\d{2})$/i.test(f))
      .map(f => path.join(input, f));
    rarFiles = files;
  } else {
    // Single file - look for related volumes
    const dir = path.dirname(input);
    const base = path.basename(input, path.extname(input));
    const files = fs.readdirSync(dir)
      .filter(f => f.startsWith(base) && /\.(rar|r\d{2})$/i.test(f))
      .map(f => path.join(dir, f));
    rarFiles = files;
  }

  if (rarFiles.length === 0) {
    console.error('No RAR files found');
    process.exit(1);
  }

  console.log(`Found ${rarFiles.length} RAR volume(s):`);
  rarFiles.forEach(f => console.log(`  ${path.basename(f)}`));
  console.log('');

  // Create LocalFileMedia for each file
  const mediaFiles = rarFiles.map(f => new LocalFileMedia(f));

  // Parse the archive
  const pkg = new RarFilesPackage(mediaFiles);
  
  console.log('Parsing RAR archive...');
  const innerFiles = await pkg.parse();

  console.log(`\nFound ${innerFiles.length} inner file(s):`);
  for (const file of innerFiles) {
    const sizeMB = (file.length / (1024 * 1024)).toFixed(2);
    console.log(`  ${file.name} (${sizeMB} MB)`);
  }

  // If there's a video file, show how to read a range
  const videoFile = innerFiles.find(f => 
    /\.(mkv|mp4|avi|mov)$/i.test(f.name)
  );
  
  if (videoFile) {
    console.log(`\nReading first 1KB of ${videoFile.name}...`);
    const chunk = await videoFile.createReadStream({ start: 0, end: 1023 });
    console.log(`Got ${chunk.length} bytes`);
    console.log(`First 16 bytes: ${chunk.slice(0, 16).toString('hex')}`);
  }
}

main().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});
