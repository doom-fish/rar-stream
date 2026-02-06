// E2E browser tests: simulate user uploading RAR files and decompressing via the UI
// Run with: npx playwright test test-browser/e2e.test.ts

import { test, expect } from '@playwright/test';
import path from 'path';

const fixturesDir = path.resolve(__dirname, '../__fixtures__');

async function uploadAndDecompress(page, filePath: string) {
  await page.goto('http://localhost:8765/test-browser/index.html');
  await page.waitForFunction(
    () => document.querySelector('.test.pass') !== null,
    { timeout: 10000 },
  );

  const fileInput = page.locator('#fileInput');
  await fileInput.setInputFiles(filePath);
  await page.click('button');

  // Wait for output to contain a result indicator
  await page.waitForFunction(
    () => {
      const output = document.getElementById('output')?.textContent ?? '';
      return (
        output.includes('✓') ||
        output.includes('ERROR') ||
        output.includes('Not a valid')
      );
    },
    { timeout: 15000 },
  );

  return page.locator('#output').textContent();
}

test.describe('E2E: User uploads and decompresses RAR files', () => {
  test('RAR4 compressed file → decompress → verify output', async ({
    page,
  }) => {
    const output = await uploadAndDecompress(
      page,
      path.join(fixturesDir, 'compressed/lipsum_rar4_default.rar'),
    );

    expect(output).toContain('Is RAR archive: true');
    expect(output).toContain('RAR version: RAR4');
    expect(output).toContain('File name: lorem_ipsum.txt');
    expect(output).toContain('✓ Decompressed: 3515 bytes');
    expect(output).toContain('Lorem ipsum');
  });

  test('RAR4 stored file → shows stored message', async ({ page }) => {
    const output = await uploadAndDecompress(
      page,
      path.join(fixturesDir, 'compressed/lipsum_rar4_store.rar'),
    );

    expect(output).toContain('Is RAR archive: true');
    expect(output).toContain('RAR version: RAR4');
    expect(output).toContain('Store (no compression)');
    expect(output).toContain('✓ File is stored');
  });

  test('RAR4 PPMd compressed file → decompress → verify output', async ({
    page,
  }) => {
    const output = await uploadAndDecompress(
      page,
      path.join(fixturesDir, 'compressed/lipsum_rar4_ppmd.rar'),
    );

    expect(output).toContain('Is RAR archive: true');
    expect(output).toContain('✓ Decompressed: 3515 bytes');
    expect(output).toContain('Lorem ipsum');
  });

  test('RAR5 compressed file → decompress → verify output', async ({
    page,
  }) => {
    const output = await uploadAndDecompress(
      page,
      path.join(fixturesDir, 'rar5/compressed.rar'),
    );

    expect(output).toContain('Is RAR archive: true');
    expect(output).toContain('RAR version: RAR5');
    expect(output).toContain('File name: compress_test.txt');
    expect(output).toContain('✓ Decompressed: 152 bytes');
    expect(output).toContain('This is a test file');
  });

  test('RAR5 stored file → shows stored message', async ({ page }) => {
    const output = await uploadAndDecompress(
      page,
      path.join(fixturesDir, 'rar5/stored.rar'),
    );

    expect(output).toContain('Is RAR archive: true');
    expect(output).toContain('RAR version: RAR5');
    expect(output).toContain('Store (no compression)');
    expect(output).toContain('✓ File is stored');
  });

  test('non-RAR file → shows error', async ({ page }) => {
    const output = await uploadAndDecompress(
      page,
      path.join(fixturesDir, 'single/single.txt'),
    );

    expect(output).toContain('Not a valid RAR archive');
  });
});
