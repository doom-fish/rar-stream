#!/usr/bin/env node
/**
 * Test WebTorrent + rar-stream integration locally
 * 
 * Creates a torrent from a local RAR file and tests the streaming flow.
 * This is a self-contained test that doesn't require internet access.
 */

import WebTorrent from 'webtorrent';
import path from 'path';
import { fileURLToPath } from 'url';
import { RarFilesPackage } from '../lib/index.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rarPath = path.resolve(__dirname, '../__fixtures__/single/single.rar');

console.log('=== WebTorrent + rar-stream Local Test ===\n');

// Create two WebTorrent clients - one to seed, one to download
const seeder = new WebTorrent();
const leecher = new WebTorrent();

/**
 * Wraps a WebTorrent file to implement the FileMedia interface
 */
function wrapTorrentFile(torrentFile) {
  return {
    get name() { return torrentFile.name; },
    get length() { return torrentFile.length; },
    createReadStream({ start, end }) {
      return new Promise((resolve, reject) => {
        const stream = torrentFile.createReadStream({ start, end });
        const chunks = [];
        stream.on('data', chunk => chunks.push(chunk));
        stream.on('end', () => resolve(Buffer.concat(chunks)));
        stream.on('error', reject);
      });
    },
    getReadableStream(opts) {
      return torrentFile.createReadStream(opts);
    },
  };
}

console.log(`1. Seeding RAR file: ${rarPath}`);

// Seed the RAR file
seeder.seed(rarPath, { announceList: [] }, (torrent) => {
  console.log(`   Torrent created: ${torrent.infoHash}`);
  console.log(`   Magnet URI: ${torrent.magnetURI.slice(0, 60)}...`);
  
  console.log('\n2. Leecher connecting to seeder...');
  
  // Leecher downloads from seeder
  leecher.add(torrent.magnetURI, { path: '/tmp/webtorrent-test' }, async (downloadedTorrent) => {
    console.log(`   Connected! Files: ${downloadedTorrent.files.length}`);
    
    // Find RAR files
    const rarFiles = downloadedTorrent.files.filter(f => f.name.endsWith('.rar'));
    console.log(`   RAR files found: ${rarFiles.length}`);
    
    if (rarFiles.length === 0) {
      console.log('   No RAR files in torrent');
      cleanup();
      return;
    }
    
    console.log('\n3. Parsing RAR archive via rar-stream...');
    
    // Wrap torrent files for rar-stream (custom FileMedia)
    const wrappedFiles = rarFiles.map(wrapTorrentFile);
    const pkg = new RarFilesPackage(wrappedFiles);
    
    try {
      const innerFiles = await pkg.parse();
      
      console.log(`   Found ${innerFiles.length} file(s) inside RAR:`);
      innerFiles.forEach(f => {
        console.log(`     - ${f.name} (${f.length} bytes)`);
      });
      
      console.log('\n4. Reading file content via getReadableStream...');
      
      const file = innerFiles[0];
      const stream = file.getReadableStream();
      const chunks = [];
      
      for await (const chunk of stream) {
        chunks.push(chunk);
      }
      
      const content = Buffer.concat(chunks);
      console.log(`   Read ${content.length} bytes`);
      console.log(`   First 50 chars: "${content.toString('utf8', 0, 50)}..."`);
      
      console.log('\n✅ WebTorrent + rar-stream integration works!\n');
      
    } catch (err) {
      console.error('   Error:', err.message);
    }
    
    cleanup();
  });
});

function cleanup() {
  console.log('Cleaning up...');
  seeder.destroy();
  leecher.destroy();
  process.exit(0);
}

// Timeout after 30 seconds
setTimeout(() => {
  console.log('\n⏰ Timeout - test took too long');
  cleanup();
  process.exit(1);
}, 30000);
