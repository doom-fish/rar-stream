import { expect, test } from "vitest";
import { Duplex } from "stream";
import { streamToBuffer, bufferToStream } from "./stream-utils.js";

test("streamToBuffer is a function", () =>
  expect(
    typeof streamToBuffer === "function",
    "streamToBuffer is not a function"
  ));

test("bufferToStream is a function", () =>
  expect(
    typeof bufferToStream === "function",
    "bufferToStream is a function"
  ).toBeTruthy());
test("bufferToStream returns a Readable stream", () =>
  expect(
    bufferToStream() instanceof Duplex,
    "bufferToStream does not return a stream"
  ).toBeTruthy());
test("stream to buffer conversion works both ways", async () => {
  const bufferContent = "bufferString1234";
  const buffer = Buffer.from(bufferContent);
  const stream = bufferToStream(buffer);
  const readBuffer = await streamToBuffer(stream);
  expect(buffer).toEqual(readBuffer);
  expect(bufferContent).toBe(readBuffer.toString("utf-8"));
});
