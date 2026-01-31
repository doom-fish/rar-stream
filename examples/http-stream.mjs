#!/usr/bin/env node
/**
 * HTTP Video Streaming from RAR Archive
 * 
 * Serves video files from a RAR archive with HTTP range request support.
 * Works with any video player that supports HTTP streaming (VLC, browsers, etc.)
 * 
 * Usage:
 *   node examples/http-stream.mjs <path-to-rar-file>
 * 
 * Example:
 *   node examples/http-stream.mjs ./movie.rar
 *   # Then open http://localhost:8080 in VLC or browser
 */

import http from 'http';
import path from 'path';
import { LocalFileMedia, RarFilesPackage } from '../lib/index.mjs';

const rarPath = process.argv[2];

if (!rarPath) {
  console.log('Usage: node examples/http-stream.mjs <path-to-rar-file>');
  console.log('');
  console.log('This starts an HTTP server that streams video from a RAR archive.');
  console.log('Supports HTTP range requests for seeking in video players.');
  process.exit(1);
}

async function main() {
  console.log(`Opening RAR archive: ${rarPath}`);
  
  // Open the RAR file
  const media = new LocalFileMedia(path.resolve(rarPath));
  const pkg = new RarFilesPackage([media]);
  
  console.log('Parsing archive...');
  const files = await pkg.parse();
  
  console.log(`\nFound ${files.length} file(s):`);
  files.forEach((f, i) => {
    const sizeMB = (f.length / 1024 / 1024).toFixed(1);
    console.log(`  [${i}] ${f.name} (${sizeMB} MB)`);
  });
  
  // Find video files
  const videos = files.filter(f => 
    /\.(mkv|mp4|avi|mov|wmv|webm|m4v)$/i.test(f.name)
  );
  
  if (videos.length === 0) {
    console.log('\nNo video files found. Serving first file instead.');
  }
  
  const fileToServe = videos[0] || files[0];
  console.log(`\nServing: ${fileToServe.name}`);
  
  // Determine content type
  const ext = fileToServe.name.split('.').pop()?.toLowerCase();
  const contentTypes = {
    mkv: 'video/x-matroska',
    mp4: 'video/mp4',
    m4v: 'video/mp4',
    avi: 'video/x-msvideo',
    mov: 'video/quicktime',
    wmv: 'video/x-ms-wmv',
    webm: 'video/webm',
    txt: 'text/plain',
    pdf: 'application/pdf',
  };
  const contentType = contentTypes[ext] || 'application/octet-stream';
  
  // Start HTTP server
  const PORT = 8080;
  const server = http.createServer((req, res) => {
    const range = req.headers.range;
    const fileSize = fileToServe.length;
    
    console.log(`${new Date().toISOString()} ${req.method} ${req.url} ${range || 'full'}`);
    
    if (range) {
      // Parse range header
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
      
      // Stream the range using getReadableStream
      const stream = fileToServe.getReadableStream({ start, end });
      stream.pipe(res);
      
    } else {
      // Full file request
      res.writeHead(200, {
        'Content-Length': fileSize,
        'Content-Type': contentType,
        'Accept-Ranges': 'bytes',
      });
      
      // Stream entire file
      const stream = fileToServe.getReadableStream();
      stream.pipe(res);
    }
  });
  
  server.listen(PORT, () => {
    console.log(`\n🎬 Streaming server started!`);
    console.log(`   URL: http://localhost:${PORT}/`);
    console.log(`   File: ${fileToServe.name}`);
    console.log(`   Size: ${(fileToServe.length / 1024 / 1024).toFixed(1)} MB`);
    console.log(`\n   Open in VLC: vlc http://localhost:${PORT}/`);
    console.log(`   Press Ctrl+C to stop\n`);
  });
}

main().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});
