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
 */

import WebTorrent from 'webtorrent';
import http from 'http';
import { RarFilesPackage } from '../lib/index.mjs';

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

client.add(torrentId, { path: '/tmp/webtorrent' }, async (torrent) => {
  console.log(`Torrent: ${torrent.name}`);
  console.log(`Files: ${torrent.files.length}`);
  
  // Find RAR files in the torrent (includes .rar, .r00, .r01, etc.)
  const rarFiles = torrent.files
    .filter(f => /\.(rar|r\d{2})$/i.test(f.name))
    .sort((a, b) => a.name.localeCompare(b.name));
  
  if (rarFiles.length === 0) {
    console.error('No RAR files found in torrent');
    console.log('\nAvailable files:');
    torrent.files.forEach(f => console.log(`  - ${f.name}`));
    process.exit(1);
  }
  
  console.log(`Found ${rarFiles.length} RAR file(s):`);
  rarFiles.forEach(f => console.log(`  ${f.name} (${(f.length / 1024 / 1024).toFixed(1)} MB)`));
  
  // WebTorrent files already implement the FileMedia interface!
  // No wrapper needed - they have name, length, and createReadStream
  console.log('\nParsing RAR archive...');
  const pkg = new RarFilesPackage(rarFiles);
  
  try {
    const innerFiles = await pkg.parse();
    
    console.log(`\nFound ${innerFiles.length} file(s) inside RAR:`);
    innerFiles.forEach(f => {
      console.log(`  ${f.name} (${(f.length / 1024 / 1024).toFixed(1)} MB)`);
    });
    
    // Find a video file
    const video = innerFiles.find(f => 
      /\.(mkv|mp4|avi|mov|wmv|webm)$/i.test(f.name)
    );
    
    if (!video) {
      console.log('\nNo video files found in the RAR archive.');
      console.log('Available files:');
      innerFiles.forEach(f => console.log(`  - ${f.name}`));
      process.exit(0);
    }
    
    console.log(`\nSelected video: ${video.name}`);
    console.log(`Size: ${(video.length / 1024 / 1024).toFixed(1)} MB`);
    
    // Determine content type from extension
    const ext = video.name.split('.').pop()?.toLowerCase();
    const contentTypes = {
      mkv: 'video/x-matroska',
      mp4: 'video/mp4',
      avi: 'video/x-msvideo',
      mov: 'video/quicktime',
      wmv: 'video/x-ms-wmv',
      webm: 'video/webm',
    };
    const contentType = contentTypes[ext] || 'application/octet-stream';
    
    // Start HTTP server for streaming
    const PORT = 8080;
    const server = http.createServer((req, res) => {
      const range = req.headers.range;
      const fileSize = video.length;
      
      console.log(`${new Date().toISOString()} ${req.method} ${req.url} ${range || 'full'}`);
      
      if (range) {
        // Handle range request
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
        
        // Stream the range from the RAR archive
        const stream = video.createReadStream({ start, end });
        stream.pipe(res);
      } else {
        // Full file request
        res.writeHead(200, {
          'Content-Length': fileSize,
          'Content-Type': contentType,
          'Accept-Ranges': 'bytes',
        });
        
        const stream = video.createReadStream();
        stream.pipe(res);
      }
    });
    
    server.listen(PORT, () => {
      console.log(`\n🎬 Video streaming server started!`);
      console.log(`   Open in VLC: vlc http://localhost:${PORT}/`);
      console.log(`   Press Ctrl+C to stop\n`);
    });
    
  } catch (err) {
    console.error('Error parsing RAR:', err.message);
    process.exit(1);
  }
});

client.on('error', (err) => {
  console.error('WebTorrent error:', err.message);
  process.exit(1);
});
