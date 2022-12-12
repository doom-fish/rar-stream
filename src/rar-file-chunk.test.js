import { expect, test } from "vitest";
import { streamToBuffer } from "./stream-utils";
import { MockFileMedia } from "./parsing/__mocks__/mock-file-media";
import { RarFileChunk } from "./rar-file-chunk";

test("RarFileChunk#getStream should return a stream from its FileMedia", async () => {
  const bufferString = "123456789A";
  const fileMedia = new MockFileMedia(bufferString);
  const rarFileChunk = new RarFileChunk(fileMedia, 0, 5);
  const stream = await rarFileChunk.getStream();
  const buffer = await streamToBuffer(stream);
  expect(Buffer.from(bufferString, "hex")).toEqual(buffer);
});

test("RarFileChunk#getStream should return a stream with a subset stream of FileMedia", async () => {
  const bufferString = "123456789A";
  const fileMedia = new MockFileMedia(bufferString);
  const rarFileChunk = new RarFileChunk(fileMedia, 2, 5);
  const stream = await rarFileChunk.getStream();
  const buffer = await streamToBuffer(stream);
  expect(Buffer.from("56789A", "hex")).toEqual(buffer);
});

test("RarFileChunk#getStream should return a stream with another subset stream of FileMedia", async () => {
  const bufferString = "123456789A";
  const fileMedia = new MockFileMedia(bufferString);
  const rarFileChunk = new RarFileChunk(fileMedia, 1, 3);
  const stream = await rarFileChunk.getStream();
  const buffer = await streamToBuffer(stream);
  expect(Buffer.from("3456", "hex")).toEqual(buffer);
});

test("RarFileChunk#length should return end - start offset", () => {
  const bufferString = "123456789A";
  const fileMedia = new MockFileMedia(bufferString);
  let rarFileChunk = new RarFileChunk(fileMedia, 1, 3);
  expect(rarFileChunk.length).toBe(2);
  rarFileChunk = new RarFileChunk(fileMedia, 0, 3);
  expect(rarFileChunk.length).toBe(3);
  rarFileChunk = new RarFileChunk(fileMedia, 1, 2);
  expect(rarFileChunk.length).toBe(1);
  rarFileChunk = new RarFileChunk(fileMedia, 0, 5);
  expect(rarFileChunk.length).toBe(5);
});
