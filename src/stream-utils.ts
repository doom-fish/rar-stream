import { Stream, Duplex } from "stream";

export const streamToBuffer = async (stream: Stream | NodeJS.ReadableStream): Promise<Buffer> =>
  new Promise((resolve, reject) => {
    const buffers: Uint8Array[] = [];
    stream.on("error", reject);
    stream.on("data", (data) => buffers.push(data));
    stream.on("end", () => resolve(Buffer.concat(buffers)));
  });

export const bufferToStream = (buffer: Buffer): Stream => {
  const stream = new Duplex();
  stream.push(buffer);
  stream.push(null);
  return stream;
};
