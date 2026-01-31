// TypeScript tests for rar-stream NAPI module
// These tests validate the Rust implementation matches the original JS behavior

import { expect, test, describe } from "vitest";
import path from "path";
import fs from "fs";
import { Readable } from "stream";

// Import from the wrapper module with stream support
import { RarFilesPackage, LocalFileMedia, InnerFile } from "./lib/index.mjs";

const fixturePath = path.resolve(__dirname, "./__fixtures__");

// Helper to read all files to buffers
const readToEnd = (files: InnerFile[]) =>
  Promise.all(files.map((file: InnerFile) => file.readToEnd()));

// File paths
const singleFilePath = path.resolve(fixturePath, "single/single.txt");
const multiFilePath = path.resolve(fixturePath, "multi/multi.txt");
const singleSplitted1FilePath = path.resolve(fixturePath, "single-splitted/splitted1.txt");
const singleSplitted2FilePath = path.resolve(fixturePath, "single-splitted/splitted2.txt");
const singleSplitted3FilePath = path.resolve(fixturePath, "single-splitted/splitted3.txt");
const multiSplitted1FilePath = path.resolve(fixturePath, "multi-splitted/splitted1.txt");
const multiSplitted2FilePath = path.resolve(fixturePath, "multi-splitted/splitted2.txt");
const multiSplitted3FilePath = path.resolve(fixturePath, "multi-splitted/splitted3.txt");
const multiSplitted4FilePath = path.resolve(fixturePath, "multi-splitted/splitted4.txt");

// RAR file arrays
const singleFileRarWithOneInnerFile = [
  path.resolve(fixturePath, "single/single.rar"),
].map((a) => new LocalFileMedia(a));

const singleRarWithManyInnerFiles = [
  path.resolve(fixturePath, "single-splitted/single-splitted.rar"),
].map((a) => new LocalFileMedia(a));

const multipleRarFileWithOneInnerFile = [
  path.resolve(fixturePath, "multi/multi.rar"),
  path.resolve(fixturePath, "multi/multi.r01"),
  path.resolve(fixturePath, "multi/multi.r00"),
].map((a) => new LocalFileMedia(a));

const multipleRarFileWithManyInnerFiles = [
  path.resolve(fixturePath, "multi-splitted/multi-splitted.rar"),
  path.resolve(fixturePath, "multi-splitted/multi-splitted.r00"),
  path.resolve(fixturePath, "multi-splitted/multi-splitted.r01"),
].map((a) => new LocalFileMedia(a));

describe("LocalFileMedia", () => {
  test("can read file properties", () => {
    const media = new LocalFileMedia(singleFilePath);
    expect(media.name).toBe("single.txt");
    expect(media.length).toBeGreaterThan(0);
  });

  test("can read file range", async () => {
    const media = new LocalFileMedia(singleFilePath);
    const stream = media.createReadStream({ start: 0, end: 10 });
    
    const chunks: Buffer[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    const buffer = Buffer.concat(chunks);
    expect(buffer.length).toBe(11); // Inclusive range
  });

  test("createReadStream returns a Node.js Readable stream", async () => {
    const media = new LocalFileMedia(singleFilePath);
    const stream = media.createReadStream({ start: 0, end: 99 });
    
    expect(stream).toBeInstanceOf(Readable);
    
    // Collect stream data
    const chunks: Buffer[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    const buffer = Buffer.concat(chunks);
    
    expect(buffer.length).toBe(100);
  });
});

describe("RarFilesPackage - Single RAR with one inner file", () => {
  test("can be read as whole", async () => {
    const rarPackage = new RarFilesPackage(singleFileRarWithOneInnerFile);
    const files = await rarPackage.parse();
    const [rarFileContent] = await readToEnd(files);
    const singleFileContent = fs.readFileSync(singleFilePath);

    expect(rarFileContent?.length).toBe(singleFileContent.length);
    expect(Buffer.compare(rarFileContent, singleFileContent)).toBe(0);
  });

  test("can be read in parts via stream", async () => {
    const interval = { start: 53, end: 1000 };

    const rarPackage = new RarFilesPackage(singleFileRarWithOneInnerFile);
    const [file] = await rarPackage.parse();
    const stream = file?.createReadStream(interval);
    
    // Collect stream data
    const chunks: Buffer[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    const rarFileBuffer = Buffer.concat(chunks);
    
    const singleFileBuffer = fs.readFileSync(singleFilePath).subarray(interval.start, interval.end + 1);

    expect(rarFileBuffer.length).toBe(singleFileBuffer.length);
    expect(Buffer.compare(rarFileBuffer, singleFileBuffer)).toBe(0);
  });

  test("createReadStream returns a Node.js Readable stream", async () => {
    const rarPackage = new RarFilesPackage(singleFileRarWithOneInnerFile);
    const [file] = await rarPackage.parse();
    
    // Stream entire file
    const stream = file.createReadStream();
    expect(stream).toBeInstanceOf(Readable);
    
    const chunks: Buffer[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    const buffer = Buffer.concat(chunks);
    
    const singleFileContent = fs.readFileSync(singleFilePath);
    expect(buffer.length).toBe(singleFileContent.length);
    expect(Buffer.compare(buffer, singleFileContent)).toBe(0);
  });

  test("createReadStream supports byte range", async () => {
    const rarPackage = new RarFilesPackage(singleFileRarWithOneInnerFile);
    const [file] = await rarPackage.parse();
    
    // Stream a range
    const stream = file.createReadStream({ start: 100, end: 199 });
    
    const chunks: Buffer[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    const buffer = Buffer.concat(chunks);
    
    const singleFileContent = fs.readFileSync(singleFilePath);
    const expected = singleFileContent.subarray(100, 200);
    
    expect(buffer.length).toBe(100);
    expect(Buffer.compare(buffer, expected)).toBe(0);
  });
});

describe("RarFilesPackage - Single RAR with many inner files", () => {
  test("can be read as whole", async () => {
    const rarPackage = new RarFilesPackage(singleRarWithManyInnerFiles);
    const files = await rarPackage.parse();
    const buffers = await readToEnd(files);

    const splitted1 = fs.readFileSync(singleSplitted1FilePath);
    const splitted2 = fs.readFileSync(singleSplitted2FilePath);
    const splitted3 = fs.readFileSync(singleSplitted3FilePath);

    // Find matching files by name
    const findByName = (name: string) => {
      const idx = files.findIndex((f: InnerFile) => f.name.includes(name));
      return idx >= 0 ? buffers[idx] : null;
    };

    const rarFile1 = findByName("splitted1");
    const rarFile2 = findByName("splitted2");
    const rarFile3 = findByName("splitted3");

    expect(rarFile1?.length).toBe(splitted1.length);
    expect(rarFile2?.length).toBe(splitted2.length);
    expect(rarFile3?.length).toBe(splitted3.length);
  });
});

describe("RarFilesPackage - Multiple RAR with one inner file", () => {
  test("can be read as whole", async () => {
    const rarPackage = new RarFilesPackage(multipleRarFileWithOneInnerFile);
    const [rarFileBuffer] = await rarPackage.parse().then(readToEnd);
    const multiFile = fs.readFileSync(multiFilePath);
    
    expect(rarFileBuffer?.length).toBe(multiFile.length);
    expect(Buffer.compare(rarFileBuffer ?? Buffer.alloc(0), multiFile)).toBe(0);
  });

  test("can be read in parts via stream", async () => {
    const interval = { start: 0, end: 100 };

    const rarPackage = new RarFilesPackage(multipleRarFileWithOneInnerFile);
    const files = await rarPackage.parse();
    const file = files[0];
    expect(file).toBeDefined();
    const stream = file.createReadStream(interval);
    
    const chunks: Buffer[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    const rarFileBuffer = Buffer.concat(chunks);
    const multiFileBuffer = fs.readFileSync(multiFilePath).subarray(interval.start, interval.end + 1);

    expect(rarFileBuffer.length).toBe(multiFileBuffer.length);
    expect(Buffer.compare(rarFileBuffer, multiFileBuffer)).toBe(0);
  });
});

describe("RarFilesPackage - Multiple RAR with many inner files", () => {
  test("can be read as whole", async () => {
    const rarPackage = new RarFilesPackage(multipleRarFileWithManyInnerFiles);
    const files = await rarPackage.parse();
    const buffers = await readToEnd(files);

    const splitted1 = fs.readFileSync(multiSplitted1FilePath);
    const splitted2 = fs.readFileSync(multiSplitted2FilePath);
    const splitted3 = fs.readFileSync(multiSplitted3FilePath);
    const splitted4 = fs.readFileSync(multiSplitted4FilePath);

    // Find matching files by name
    const findByName = (name: string) => {
      const idx = files.findIndex((f: InnerFile) => f.name.includes(name));
      return idx >= 0 ? buffers[idx] : null;
    };

    const rarFile1 = findByName("splitted1");
    const rarFile2 = findByName("splitted2");
    const rarFile3 = findByName("splitted3");
    const rarFile4 = findByName("splitted4");

    expect(rarFile1?.length).toBe(splitted1.length);
    expect(rarFile2?.length).toBe(splitted2.length);
    expect(rarFile3?.length).toBe(splitted3.length);
    expect(rarFile4?.length).toBe(splitted4.length);
  });
});

describe("ParseOptions", () => {
  test("maxFiles limits results", async () => {
    const rarPackage = new RarFilesPackage(singleRarWithManyInnerFiles);
    const files = await rarPackage.parse({ maxFiles: 1 });
    expect(files.length).toBe(1);
  });
});

describe("Compression Support", () => {
  const compressedPath = path.resolve(fixturePath, "compressed");
  const expectedContent = fs.readFileSync(
    path.resolve(compressedPath, "lorem_ipsum.txt.expected")
  );

  test("LZSS store (0x30) - no compression", async () => {
    const media = new LocalFileMedia(
      path.resolve(compressedPath, "lipsum_rar4_store.rar")
    );
    const pkg = new RarFilesPackage([media]);
    const [file] = await pkg.parse();
    const content = await file.readToEnd();
    expect(content.length).toBe(expectedContent.length);
    expect(Buffer.compare(content, expectedContent)).toBe(0);
  });

  test("LZSS default (0x33)", async () => {
    const media = new LocalFileMedia(
      path.resolve(compressedPath, "lipsum_rar4_default.rar")
    );
    const pkg = new RarFilesPackage([media]);
    const [file] = await pkg.parse();
    const content = await file.readToEnd();
    expect(content.length).toBe(expectedContent.length);
    expect(Buffer.compare(content, expectedContent)).toBe(0);
  });

  test("LZSS max (0x35)", async () => {
    const media = new LocalFileMedia(
      path.resolve(compressedPath, "lipsum_rar4_max.rar")
    );
    const pkg = new RarFilesPackage([media]);
    const [file] = await pkg.parse();
    const content = await file.readToEnd();
    expect(content.length).toBe(expectedContent.length);
    expect(Buffer.compare(content, expectedContent)).toBe(0);
  });

  test("Delta filter", async () => {
    const media = new LocalFileMedia(
      path.resolve(compressedPath, "lipsum_rar4_delta.rar")
    );
    const pkg = new RarFilesPackage([media]);
    const [file] = await pkg.parse();
    const content = await file.readToEnd();
    expect(content.length).toBe(expectedContent.length);
    expect(Buffer.compare(content, expectedContent)).toBe(0);
  });

  test("PPMd compression", async () => {
    const media = new LocalFileMedia(
      path.resolve(compressedPath, "lipsum_rar4_ppmd.rar")
    );
    const pkg = new RarFilesPackage([media]);
    const [file] = await pkg.parse();
    const content = await file.readToEnd();
    expect(content.length).toBe(expectedContent.length);
    expect(Buffer.compare(content, expectedContent)).toBe(0);
  });
});

describe("RAR5 Support", () => {
  const rar5Path = path.resolve(fixturePath, "rar5");

  test("RAR5 stored (method 0)", async () => {
    const media = new LocalFileMedia(path.resolve(rar5Path, "stored.rar"));
    const pkg = new RarFilesPackage([media]);
    const files = await pkg.parse();
    expect(files.length).toBe(1);
    expect(files[0].name).toBe("stored_test.txt");
    const content = await files[0].readToEnd();
    const text = content.toString("utf-8");
    expect(text).toContain("Hello stored RAR5!");
  });

  test("RAR5 compressed (method 3)", async () => {
    const media = new LocalFileMedia(path.resolve(rar5Path, "compressed.rar"));
    const pkg = new RarFilesPackage([media]);
    const files = await pkg.parse();
    expect(files.length).toBe(1);
    expect(files[0].name).toBe("compress_test.txt");
    expect(files[0].length).toBe(152); // Unpacked size
    const content = await files[0].readToEnd();
    expect(content.length).toBe(152);
    const text = content.toString("utf-8");
    expect(text).toContain("This is a test file");
    expect(text).toContain("hello hello");
  });
});
