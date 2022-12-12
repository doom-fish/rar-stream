import { expect, test } from "vitest";

import { streamToBuffer, bufferToStream } from "./stream-utils.js";

test("stream to buffer conversion works both ways", async () => {
  const bufferContent = "bufferString1234";
  const buffer = Buffer.from(bufferContent);
  const stream = bufferToStream(buffer);
  const readBuffer = await streamToBuffer(stream);
  expect(buffer).toEqual(readBuffer);
  expect(bufferContent).toBe(readBuffer.toString("utf-8"));
});
