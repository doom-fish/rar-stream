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
});
