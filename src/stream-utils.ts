import { Duplex, Readable } from "stream";

export const streamToBuffer = async (stream: Readable): Promise<Buffer> =>
  new Promise((resolve, reject) => {
    const buffers: Uint8Array[] = [];
    stream.on("error", reject);
    stream.on("data", (data) => buffers.push(data));
    stream.on("end", () => resolve(Buffer.concat(buffers)));
  });

export const bufferToStream = (buffer: Buffer): Duplex => {
  const stream = new Duplex();
  stream.push(buffer);
  stream.push(null);
  return stream;
};
