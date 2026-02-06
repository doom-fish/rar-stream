#!/usr/bin/env npx tsx
// Parse a RAR archive, list inner files, and read content.
// Usage: npx tsx examples/basic.ts <path-to-rar-file-or-directory>

import { RarFilesPackage, LocalFileMedia } from '../index.js';
import fs from 'fs';
import path from 'path';

async function main(): Promise<void> {
  const input = process.argv[2];

  if (!input) {
    console.log('Usage: npx tsx examples/basic.ts <rar-file-or-directory>');
    console.log('  npx tsx examples/basic.ts ./archive.rar');
    console.log('  npx tsx examples/basic.ts ./multi-volume-dir/');
    process.exit(1);
  }

  // Collect RAR volumes
  let rarPaths: string[];
  const stat = fs.statSync(input);
  if (stat.isDirectory()) {
    rarPaths = fs
      .readdirSync(input)
      .filter((f) => /\.(rar|r\d{2})$/i.test(f))
      .map((f) => path.join(input, f));
  } else {
    const dir = path.dirname(input);
    const base = path.basename(input, path.extname(input));
    rarPaths = fs
      .readdirSync(dir)
      .filter((f) => f.startsWith(base) && /\.(rar|r\d{2})$/i.test(f))
      .map((f) => path.join(dir, f));
  }

  if (rarPaths.length === 0) {
    console.error('No RAR files found');
    process.exit(1);
  }

  console.log(`Found ${rarPaths.length} volume(s):`);
  for (const f of rarPaths) console.log(`  ${path.basename(f)}`);

  const mediaFiles = rarPaths.map((f) => new LocalFileMedia(f));
  const pkg = new RarFilesPackage(mediaFiles);
  const innerFiles = await pkg.parse();

  console.log(`\n${innerFiles.length} inner file(s):`);
  for (const file of innerFiles) {
    const sizeMB = (file.length / (1024 * 1024)).toFixed(2);
    console.log(`  ${file.name} (${sizeMB} MB)`);
  }

  // Read first file content
  if (innerFiles.length > 0) {
    const first = innerFiles[0];
    const buf = await first.readToEnd();
    console.log(`\nRead ${first.name}: ${buf.length} bytes`);

    // Show preview if text
    const preview = buf.subarray(0, 200).toString('utf8');
    if (/^[\x20-\x7E\n\r\t]+$/.test(preview)) {
      console.log(`Preview: ${preview.slice(0, 100)}...`);
    }
  }
}

main().catch((err) => {
  console.error('Error:', err.message);
  process.exit(1);
});
