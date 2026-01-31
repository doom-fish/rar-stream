#!/usr/bin/env node
// Test parsing RAR header from the first bytes of an archive
// Usage: node test-header.mjs <rar-file>

import { parseRarHeader, isRarArchive } from './index.js';
import fs from 'fs';

const file = process.argv[2];
if (!file) {
  console.log('Usage: node test-header.mjs <rar-file>');
  process.exit(1);
}

// Read first 500 bytes
const buffer = Buffer.alloc(500);
const fd = fs.openSync(file, 'r');
fs.readSync(fd, buffer, 0, 500, 0);
fs.closeSync(fd);

console.log('Is RAR archive:', isRarArchive(buffer));
console.log('First 20 bytes:', buffer.slice(0, 20).toString('hex'));

const info = parseRarHeader(buffer);
if (info) {
  console.log('Inner file info:');
  console.log('  Name:', info.name);
  console.log('  Packed size:', (info.packedSize / 1024 / 1024).toFixed(2), 'MB');
  console.log('  Unpacked size:', (info.unpackedSize / 1024 / 1024 / 1024).toFixed(2), 'GB');
  console.log('  Method:', info.method === 0x30 ? 'Store (no compression)' : `0x${info.method.toString(16)}`);
  console.log('  Continues in next:', info.continuesInNext);
} else {
  console.log('Failed to parse RAR header');
}
