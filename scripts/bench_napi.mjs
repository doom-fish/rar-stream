#!/usr/bin/env node
// Benchmark NAPI decompression performance
// Usage: node scripts/bench_napi.mjs [archive.rar]

import { RarFilesPackage, LocalFileMedia } from '../lib/index.mjs';
import { createHash } from 'crypto';
import { execSync } from 'child_process';
import { existsSync } from 'fs';

const defaultArchive = '__fixtures__/large/alpine-200mb.rar';
const archive = process.argv[2] || defaultArchive;

if (!existsSync(archive)) {
  console.error(`Archive not found: ${archive}`);
  process.exit(1);
}

async function benchNapi(path, runs = 5) {
  const times = [];
  let size = 0;
  let md5 = '';

  for (let i = 0; i < runs; i++) {
    const media = new LocalFileMedia(path);
    const pkg = new RarFilesPackage([media]);
    
    const start = performance.now();
    const files = await pkg.parse();
    const buf = await files[0].readToEnd();
    const elapsed = performance.now() - start;
    
    times.push(elapsed);
    size = buf.length;
    if (i === 0) {
      md5 = createHash('md5').update(buf).digest('hex');
    }
  }

  times.sort((a, b) => a - b);
  const median = times[Math.floor(times.length / 2)];
  const min = times[0];
  const max = times[times.length - 1];
  const sizeMB = (size / 1024 / 1024).toFixed(1);
  const throughput = (size / 1024 / 1024 / (median / 1000)).toFixed(0);

  return { median, min, max, size, sizeMB, md5, throughput, times };
}

function benchUnrar(path, runs = 5) {
  const times = [];
  for (let i = 0; i < runs; i++) {
    const start = performance.now();
    execSync(`unrar t "${path}"`, { stdio: 'ignore' });
    const elapsed = performance.now() - start;
    times.push(elapsed);
  }
  times.sort((a, b) => a - b);
  return {
    median: times[Math.floor(times.length / 2)],
    min: times[0],
    max: times[times.length - 1],
    times,
  };
}

console.log(`Benchmarking: ${archive}`);
console.log('');

// NAPI benchmark
console.log('rar-stream (NAPI)...');
const napi = await benchNapi(archive);
console.log(`  ${napi.sizeMB} MB, md5: ${napi.md5}`);
console.log(`  median: ${napi.median.toFixed(1)}ms  [${napi.min.toFixed(1)}-${napi.max.toFixed(1)}ms]`);
console.log(`  throughput: ${napi.throughput} MB/s`);

// unrar benchmark
console.log('');
console.log('official unrar...');
const unrar = benchUnrar(archive);
console.log(`  median: ${unrar.median.toFixed(1)}ms  [${unrar.min.toFixed(1)}-${unrar.max.toFixed(1)}ms]`);

// Comparison
const ratio = (napi.median / unrar.median).toFixed(2);
const speedup = (unrar.median / napi.median).toFixed(2);
console.log('');
console.log(`ratio: ${ratio}x (${napi.median < unrar.median ? speedup + 'x faster' : speedup + 'x slower'} than unrar)`);
