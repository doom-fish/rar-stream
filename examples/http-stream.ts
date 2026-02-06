#!/usr/bin/env npx tsx
// HTTP video server that streams from a RAR archive with range request support.
// Usage: npx tsx examples/http-stream.ts <path-to-rar-file>
// Then open http://localhost:8080 in VLC or a browser.

import http from 'http';
import path from 'path';
import { LocalFileMedia, RarFilesPackage, type ReadIntervalJs } from '../index.js';

const rarPath = process.argv[2];

if (!rarPath) {
  console.log('Usage: npx tsx examples/http-stream.ts <path-to-rar-file>');
  console.log('Starts an HTTP server that streams video from a RAR archive.');
  process.exit(1);
}

const CONTENT_TYPES: Record<string, string> = {
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

async function main(): Promise<void> {
  const media = new LocalFileMedia(path.resolve(rarPath));
  const pkg = new RarFilesPackage([media]);
  const files = await pkg.parse();

  console.log(`${files.length} file(s) in archive:`);
  for (const [i, f] of files.entries()) {
    console.log(`  [${i}] ${f.name} (${(f.length / 1024 / 1024).toFixed(1)} MB)`);
  }

  const videos = files.filter((f) => /\.(mkv|mp4|avi|mov|wmv|webm|m4v)$/i.test(f.name));
  const fileToServe = videos[0] ?? files[0];
  const ext = fileToServe.name.split('.').pop()?.toLowerCase() ?? '';
  const contentType = CONTENT_TYPES[ext] ?? 'application/octet-stream';

  const PORT = 8080;
  const server = http.createServer((req, res) => {
    const range = req.headers.range;
    const fileSize = fileToServe.length;

    console.log(`${req.method} ${req.url} ${range ?? 'full'}`);

    if (range) {
      const parts = range.replace(/bytes=/, '').split('-');
      const start = parseInt(parts[0], 10);
      const end = parts[1] ? parseInt(parts[1], 10) : fileSize - 1;

      res.writeHead(206, {
        'Content-Range': `bytes ${start}-${end}/${fileSize}`,
        'Accept-Ranges': 'bytes',
        'Content-Length': end - start + 1,
        'Content-Type': contentType,
      });

      // createReadStream returns Promise<Buffer>, not a stream
      fileToServe
        .createReadStream({ start, end } satisfies ReadIntervalJs)
        .then((buf) => res.end(buf))
        .catch((err) => {
          console.error('Read error:', err.message);
          res.destroy();
        });
    } else {
      res.writeHead(200, {
        'Content-Length': fileSize,
        'Content-Type': contentType,
        'Accept-Ranges': 'bytes',
      });

      fileToServe
        .readToEnd()
        .then((buf) => res.end(buf))
        .catch((err) => {
          console.error('Read error:', err.message);
          res.destroy();
        });
    }
  });

  server.listen(PORT, () => {
    console.log(`\nStreaming: ${fileToServe.name}`);
    console.log(`URL: http://localhost:${PORT}/`);
    console.log(`VLC: vlc http://localhost:${PORT}/`);
    console.log('Press Ctrl+C to stop');
  });
}

main().catch((err) => {
  console.error('Error:', err.message);
  process.exit(1);
});
