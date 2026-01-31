#!/usr/bin/env node
/**
 * WebTorrent + rar-stream Example
 * 
 * Streams video from a RAR archive inside a torrent.
 * 
 * Usage:
 *   node examples/webtorrent-stream.mjs <magnet-uri-or-torrent-file>
 * 
 * Requirements:
 *   npm install webtorrent
 * 
 * How it works:
 *   1. Downloads torrent containing RAR files
 *   2. Reads RAR header to identify inner files
 *   3. Streams and decompresses video content
 *   4. Serves via HTTP with range request support
 */

import WebTorrent from 'webtorrent';
import http from 'http';
import { Readable } from 'stream';
import { createRequire } from 'module';

// Import rar-stream APIs
const require = createRequire(import.meta.url);
const native = require('../index.js');
const { parseRarHeader, isRarArchive, LocalFileMedia, RarFilesPackage } = native;

const torrentId = process.argv[2];

if (!torrentId) {
  console.log('Usage: node examples/webtorrent-stream.mjs <magnet-uri-or-torrent-file>');
  console.log('');
  console.log('This example:');
  console.log('  1. Downloads a torrent containing RAR files');
  console.log('  2. Parses the RAR archive to find video files');
  console.log('  3. Starts an HTTP server that streams the video');
  console.log('  4. Supports HTTP range requests for seeking');
  console.log('');
  console.log('For local testing, run: node examples/webtorrent-local-test.mjs');
  process.exit(1);
}

console.log('Starting WebTorrent client...');
const client = new WebTorrent();

/**
 * Read data from a WebTorrent file
 */
function readTorrentFileRange(torrentFile, start, end) {
  return new Promise((resolve, reject) => {
    const stream = torrentFile.createReadStream({ start, end });
    const chunks = [];
    stream.on('data', chunk => chunks.push(chunk));
    stream.on('end', () => resolve(Buffer.concat(chunks)));
    stream.on('error', reject);
  });
}

client.add(torrentId, { path: '/tmp/webtorrent' }, async (torrent) => {
  console.log(`Torrent: ${torrent.name}`);
  console.log(`Files: ${torrent.files.length}`);
  
  // Find RAR files in the torrent
  const rarFiles = torrent.files
    .filter(f => /\.rar$/i.test(f.name))
    .sort((a, b) => a.name.localeCompare(b.name));
  
  if (rarFiles.length === 0) {
    console.error('No RAR files found in torrent');
    console.log('\nAvailable files:');
    torrent.files.forEach(f => console.log(`  - ${f.name}`));
    process.exit(1);
  }
  
  console.log(`Found ${rarFiles.length} RAR file(s):`);
  rarFiles.forEach(f => console.log(`  ${f.name} (${(f.length / 1024 / 1024).toFixed(1)} MB)`));
  
  // Read RAR header to get inner file info
  console.log('\nReading RAR header...');
  const headerData = await readTorrentFileRange(rarFiles[0], 0, 512);
  
  if (!isRarArchive(headerData)) {
    console.error('First file is not a valid RAR archive');
    process.exit(1);
  }
  
  const header = parseRarHeader(headerData);
  if (!header) {
    console.error('Failed to parse RAR header');
    process.exit(1);
  }
  
  console.log(`\nInner file: ${header.name}`);
  console.log(`  Packed: ${header.packedSize} bytes`);
  console.log(`  Unpacked: ${header.unpackedSize} bytes`);
  console.log(`  Method: 0x${header.method.toString(16)} (${header.method === 0x30 ? 'stored' : 'compressed'})`);
  
  // For stored files (method 0x30), we can stream directly
  // For compressed files, we'd need to decompress on-the-fly
  const isStored = header.method === 0x30;
  
  if (!isStored) {
    console.log('\n⚠️  File is compressed. Full streaming requires decompression.');
    console.log('   For best results, create torrents with stored RAR files (rar -m0)');
  }
  
  // Calculate data offset (after RAR headers)
  // This is approximate - proper implementation should parse full header
  const dataOffset = rarFiles[0].length - header.packedSize;
  
  // Determine content type
  const ext = header.name.split('.').pop()?.toLowerCase();
  const contentTypes = {
    mkv: 'video/x-matroska',
    mp4: 'video/mp4',
    avi: 'video/x-msvideo',
    mov: 'video/quicktime',
    webm: 'video/webm',
  };
  const contentType = contentTypes[ext] || 'application/octet-stream';
  
  // Start HTTP server
  const PORT = 8080;
  const fileSize = header.unpackedSize;
  
  const server = http.createServer(async (req, res) => {
    const range = req.headers.range;
    
    console.log(`${new Date().toISOString()} ${req.method} ${req.url} ${range || 'full'}`);
    
    if (range && isStored) {
      // Handle range request for stored files
      const parts = range.replace(/bytes=/, '').split('-');
      const start = parseInt(parts[0], 10);
      const end = parts[1] ? parseInt(parts[1], 10) : fileSize - 1;
      const chunkSize = end - start + 1;
      
      res.writeHead(206, {
        'Content-Range': `bytes ${start}-${end}/${fileSize}`,
        'Accept-Ranges': 'bytes',
        'Content-Length': chunkSize,
        'Content-Type': contentType,
      });
      
      // Stream from WebTorrent (offset by header size)
      const stream = rarFiles[0].createReadStream({
        start: dataOffset + start,
        end: dataOffset + end,
      });
      stream.pipe(res);
      
    } else {
      // Full file request
      res.writeHead(200, {
        'Content-Length': fileSize,
        'Content-Type': contentType,
        'Accept-Ranges': isStored ? 'bytes' : 'none',
      });
      
      if (isStored) {
        const stream = rarFiles[0].createReadStream({
          start: dataOffset,
          end: dataOffset + fileSize - 1,
        });
        stream.pipe(res);
      } else {
        // For compressed files, read and decompress entire file
        // (In production, you'd want streaming decompression)
        res.end('Compressed file streaming not implemented in this example');
      }
    }
  });
  
  server.listen(PORT, () => {
    console.log(`\n🎬 Video streaming server started!`);
    console.log(`   Open in VLC: vlc http://localhost:${PORT}/`);
    console.log(`   File: ${header.name}`);
    console.log(`   Size: ${(fileSize / 1024 / 1024).toFixed(1)} MB`);
    console.log(`   Range requests: ${isStored ? 'supported' : 'not supported (compressed)'}`);
    console.log(`\n   Press Ctrl+C to stop\n`);
  });
});

client.on('error', (err) => {
  console.error('WebTorrent error:', err.message);
  process.exit(1);
});
