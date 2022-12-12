//
import { expect, test } from "vitest";
import { InnerFileStream } from "./inner-file-stream.js";
import { RarFileChunk } from "./rar-file-chunk.js";
import { MockFileMedia } from "./parsing/__mocks__/mock-file-media.js";
import { streamToBuffer } from "./stream-utils.js";
test("inner file stream should stream over list of file chunks", async () => {
    const bufferString = "123456789ABC";
    const fileMedia = new MockFileMedia(bufferString);
    const rarStream = new InnerFileStream([
        new RarFileChunk(fileMedia, 0, 2),
        new RarFileChunk(fileMedia, 2, 6),
    ]);
    const buffer = await streamToBuffer(rarStream);
    expect(buffer).toEqual(Buffer.from(bufferString, "hex"));
});
test("inner file stream should stream over list of file chunks that are fragmented", async () => {
    const bufferString = "123456789ABC";
    const fragmentedResult = "349ABC";
    const fileMedia = new MockFileMedia(bufferString);
    const rarStream = new InnerFileStream([
        new RarFileChunk(fileMedia, 1, 2),
        new RarFileChunk(fileMedia, 4, 6),
    ]);
    const buffer = await streamToBuffer(rarStream);
    expect(buffer).toEqual(Buffer.from(fragmentedResult, "hex"));
});
test("inner file stream should stream over longer list of file chunks", async () => {
    const bufferString = "123456789ABC";
    const fileMedia = new MockFileMedia(bufferString);
    const rarStream = new InnerFileStream([
        new RarFileChunk(fileMedia, 0, 2),
        new RarFileChunk(fileMedia, 2, 4),
        new RarFileChunk(fileMedia, 4, 6),
    ]);
    const buffer = await streamToBuffer(rarStream);
    expect(buffer).toEqual(Buffer.from(bufferString, "hex"));
});
