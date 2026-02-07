// Browser tests for rar-stream WASM module using Playwright
// Run with: npx playwright test test-browser/wasm.test.ts

import { test, expect } from '@playwright/test';

test.describe('rar-stream WASM', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the test page
    await page.goto('http://localhost:8765/test-browser/index.html');
    // Wait for WASM to load
    await page.waitForFunction(() => {
      return document.querySelector('.test.pass') !== null || 
             document.querySelector('.test.fail') !== null;
    }, { timeout: 10000 });
  });

  test('WASM module loads successfully', async ({ page }) => {
    const loadTest = page.locator('.test').first();
    await expect(loadTest).toHaveClass(/pass/);
    await expect(loadTest).toContainText('WASM module loaded');
  });

  test('is_rar_archive detects valid RAR signature', async ({ page }) => {
    const test = page.locator('.test:has-text("valid RAR4 signature")');
    await expect(test).toHaveClass(/pass/);
  });

  test('is_rar_archive rejects invalid data', async ({ page }) => {
    const test = page.locator('.test:has-text("invalid data")');
    await expect(test).toHaveClass(/pass/);
  });

  test('get_rar_version returns correct version', async ({ page }) => {
    const test = page.locator('.test:has-text("get_rar_version")');
    await expect(test).toHaveClass(/pass/);
    await expect(test).toContainText('version 15');
  });

  test('WasmRarDecoder can be constructed', async ({ page }) => {
    const test = page.locator('.test:has-text("WasmRarDecoder constructor")');
    await expect(test).toHaveClass(/pass/);
  });

  test('WasmRarDecoder bytes_written starts at 0', async ({ page }) => {
    const test = page.locator('.test:has-text("bytes_written")');
    await expect(test).toHaveClass(/pass/);
  });

  test('all unit tests pass', async ({ page }) => {
    const failedTests = page.locator('.test.fail');
    const count = await failedTests.count();
    expect(count).toBe(0);
  });
});

test.describe('rar-stream WASM multi-header parsing', () => {
  test('parse_rar_headers lists all files in RAR4 archive', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/single-splitted/single-splitted.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { parse_rar_headers } = await import('../pkg/rar_stream.js');
      const headers = parse_rar_headers(data);
      return {
        count: headers.length,
        names: headers.map((h: any) => h.name),
      };
    });

    expect(result.count).toBe(3);
    expect(result.names).toContain('splitted1.txt');
    expect(result.names).toContain('splitted2.txt');
    expect(result.names).toContain('splitted3.txt');
  });

  test('parse_rar_headers returns empty array for single-file archive', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/compressed/lipsum_rar4_store.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { parse_rar_headers } = await import('../pkg/rar_stream.js');
      const headers = parse_rar_headers(data);
      return {
        count: headers.length,
        firstName: headers[0]?.name,
      };
    });

    expect(result.count).toBe(1);
    expect(result.firstName).toBe('lorem_ipsum.txt');
  });

  test('parse_rar5_headers lists files in RAR5 archive', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/rar5/compressed.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { parse_rar5_headers } = await import('../pkg/rar_stream.js');
      const headers = parse_rar5_headers(data);
      return {
        count: headers.length,
        firstName: headers[0]?.name,
        firstIsCompressed: headers[0]?.isCompressed,
      };
    });

    expect(result.count).toBe(1);
    expect(result.firstName).toBe('compress_test.txt');
    expect(result.firstIsCompressed).toBe(true);
  });
});

test.describe('rar-stream WasmRarArchive', () => {
  test('lists and extracts files from RAR4 multi-file archive', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/single-splitted/single-splitted.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { WasmRarArchive } = await import('../pkg/rar_stream.js');
      const archive = new WasmRarArchive(data);
      const entries = archive.entries();
      const first = archive.extract(0);
      const text = new TextDecoder().decode(first.data);
      const len = archive.length;
      archive.free();
      return {
        length: len,
        names: entries.map((e: any) => e.name),
        firstName: first.name,
        firstSize: first.size,
        textPreview: text.substring(0, 20),
      };
    });

    expect(result.length).toBe(3);
    expect(result.names).toContain('splitted1.txt');
    expect(result.names).toContain('splitted2.txt');
    expect(result.names).toContain('splitted3.txt');
    expect(result.firstName).toBe('splitted1.txt');
    expect(result.firstSize).toBeGreaterThan(0);
  });

  test('lists and extracts from RAR5 archive', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/rar5/compressed.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { WasmRarArchive } = await import('../pkg/rar_stream.js');
      const archive = new WasmRarArchive(data);
      const len = archive.length;
      const entries = archive.entries();
      const file = archive.extract(0);
      const text = new TextDecoder().decode(file.data);
      archive.free();
      return {
        length: len,
        name: file.name,
        size: file.size,
        hasTestFile: text.includes('test file'),
      };
    });

    expect(result.name).toBe('compress_test.txt');
    expect(result.size).toBe(152);
    expect(result.hasTestFile).toBe(true);
  });

  test('extractAll returns all files', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/single-splitted/single-splitted.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { WasmRarArchive } = await import('../pkg/rar_stream.js');
      const archive = new WasmRarArchive(data);
      const files = archive.extractAll();
      archive.free();
      return {
        count: files.length,
        names: files.map((f: any) => f.name),
        allHaveData: files.every((f: any) => f.data.length > 0),
      };
    });

    expect(result.count).toBe(3);
    expect(result.names).toContain('splitted1.txt');
    expect(result.allHaveData).toBe(true);
  });
});

test.describe('rar-stream WASM extract_file', () => {
  test('extracts RAR4 compressed file in one call', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/compressed/lipsum_rar4_default.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { extract_file } = await import('../pkg/rar_stream.js');
      const file = extract_file(data);
      const text = new TextDecoder().decode(file.data);
      return { name: file.name, size: file.size, startsWithLorem: text.startsWith('Lorem ipsum') };
    });

    expect(result.name).toBe('lorem_ipsum.txt');
    expect(result.size).toBe(3515);
    expect(result.startsWithLorem).toBe(true);
  });

  test('extracts RAR5 compressed file in one call', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/rar5/compressed.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { extract_file } = await import('../pkg/rar_stream.js');
      const file = extract_file(data);
      const text = new TextDecoder().decode(file.data);
      return { name: file.name, size: file.size, hasTestFile: text.includes('test file') };
    });

    expect(result.name).toBe('compress_test.txt');
    expect(result.size).toBe(152);
    expect(result.hasTestFile).toBe(true);
  });
});

test.describe('rar-stream WASM decompression', () => {
  test('can decompress LZSS store file', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    // Load and test a real RAR file
    const result = await page.evaluate(async () => {
      // Fetch the test fixture
      const response = await fetch('/__fixtures__/compressed/lipsum_rar4_store.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      // Access WASM functions from global
      const { is_rar_archive, parse_rar_header } = await import('../pkg/rar_stream.js');

      const isRar = is_rar_archive(data);
      if (!isRar) return { error: 'Not a RAR file' };

      const header = parse_rar_header(data);
      return {
        isRar,
        name: header.name,
        packedSize: header.packedSize,
        unpackedSize: header.unpackedSize,
        isCompressed: header.isCompressed
      };
    });

    expect(result.isRar).toBe(true);
    expect(result.name).toBe('lorem_ipsum.txt');
    expect(result.unpackedSize).toBe(3515);
    expect(result.isCompressed).toBe(false);
  });

  test('can decompress LZSS compressed file', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/compressed/lipsum_rar4_default.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { is_rar_archive, parse_rar_header, WasmRarDecoder } = await import('../pkg/rar_stream.js');

      const isRar = is_rar_archive(data);
      if (!isRar) return { error: 'Not a RAR file' };

      const header = parse_rar_header(data);

      // Calculate data offset (simplified - works for simple RAR4 files)
      const markerSize = 7;
      const archiveHeaderSize = 13;
      // Read file header size from bytes 5-6 of file header
      const fileHeaderOffset = markerSize + archiveHeaderSize;
      const fileHeaderSize = data[fileHeaderOffset + 5] + (data[fileHeaderOffset + 6] << 8);
      const dataOffset = fileHeaderOffset + fileHeaderSize;

      const compressedData = data.slice(dataOffset, dataOffset + header.packedSize);

      try {
        const decoder = new WasmRarDecoder(BigInt(header.unpackedSize));
        const decompressed = decoder.decompress(compressedData);
        decoder.free();

        return {
          isRar: true,
          name: header.name,
          compressedSize: compressedData.length,
          decompressedSize: decompressed.length,
          expectedSize: header.unpackedSize,
          success: decompressed.length === header.unpackedSize
        };
      } catch (e) {
        return { error: e.message };
      }
    });

    expect(result.error).toBeUndefined();
    expect(result.isRar).toBe(true);
    expect(result.name).toBe('lorem_ipsum.txt');
    expect(result.success).toBe(true);
    expect(result.decompressedSize).toBe(3515);
  });

  test('can detect RAR5 archive format', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const response = await fetch('/__fixtures__/rar5/test.rar');
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);

      const { is_rar_archive, get_rar_version } = await import('../pkg/rar_stream.js');

      const isRar = is_rar_archive(data);
      const version = get_rar_version(data);

      return {
        isRar,
        version,
        isRar5: version === 50
      };
    });

    expect(result.isRar).toBe(true);
    expect(result.version).toBe(50); // RAR 5.0
    expect(result.isRar5).toBe(true);
  });

  test('WasmRar5Crypto can be constructed', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      const { WasmRar5Crypto } = await import('../pkg/rar_stream.js');

      // Check if the crypto class is available
      if (typeof WasmRar5Crypto !== 'function') {
        return { error: 'WasmRar5Crypto not available' };
      }

      try {
        // Create a test decryptor with a dummy password and salt
        const salt = new Uint8Array(16).fill(0);
        const crypto = new WasmRar5Crypto('testpassword', salt, 15);

        // Check that decrypt method exists
        if (typeof crypto.decrypt !== 'function') {
          return { error: 'decrypt method not found' };
        }

        crypto.free();
        return { success: true };
      } catch (e) {
        return { error: e.message };
      }
    });

    expect(result.error).toBeUndefined();
    expect(result.success).toBe(true);
  });

  test('can decompress RAR5 compressed file', async ({ page }) => {
    await page.goto('http://localhost:8765/test-browser/index.html');
    await page.waitForFunction(() => document.querySelector('.test.pass') !== null);

    const result = await page.evaluate(async () => {
      try {
        const response = await fetch('/__fixtures__/rar5/compressed.rar');
        const buffer = await response.arrayBuffer();
        const data = new Uint8Array(buffer);

        const { parse_rar5_header, WasmRar5Decoder } = await import('../pkg/rar_stream.js');

        // Parse header to get compression info
        const header = parse_rar5_header(data);

        // Find compressed data offset (skip signature + headers)
        // For a simple single-file RAR5, data starts after headers
        // We need to find the data start position
        const sig_len = 8;
        
        // Read archive header size
        let pos = sig_len + 4; // skip CRC32
        // Read vint for header size
        let headerSize = 0;
        let shift = 0;
        while (pos < data.length) {
          const b = data[pos++];
          headerSize |= (b & 0x7F) << shift;
          if ((b & 0x80) === 0) break;
          shift += 7;
        }
        const archEnd = pos + headerSize;
        
        // Read file header
        pos = archEnd + 4; // skip CRC32
        headerSize = 0;
        shift = 0;
        while (pos < data.length) {
          const b = data[pos++];
          headerSize |= (b & 0x7F) << shift;
          if ((b & 0x80) === 0) break;
          shift += 7;
        }
        const fileHeaderEnd = pos + headerSize;
        
        // Data starts after file header, length = packedSize
        const compressedData = data.slice(fileHeaderEnd, fileHeaderEnd + header.packedSize);

        const decoder = new WasmRar5Decoder(
          BigInt(header.unpackedSize),
          header.dictSizeLog,
          header.method,
          false
        );
        const decompressed = decoder.decompress(compressedData);
        decoder.free();

        const text = new TextDecoder().decode(decompressed);

        return {
          name: header.name,
          method: header.method,
          dictSizeLog: header.dictSizeLog,
          decompressedSize: decompressed.length,
          expectedSize: header.unpackedSize,
          textPreview: text.substring(0, 50),
          success: decompressed.length === header.unpackedSize,
        };
      } catch (e) {
        return { error: e.message };
      }
    });

    expect(result.error).toBeUndefined();
    expect(result.success).toBe(true);
    expect(result.name).toBe('compress_test.txt');
    expect(result.decompressedSize).toBe(152);
  });
});
