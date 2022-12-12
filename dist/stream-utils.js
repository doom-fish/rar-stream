import { Duplex } from "stream";
export const streamToBuffer = async (stream) => new Promise((resolve, reject) => {
    const buffers = [];
    stream.on("error", reject);
    stream.on("data", (data) => buffers.push(data));
    stream.on("end", () => resolve(Buffer.concat(buffers)));
});
export const bufferToStream = (buffer) => {
    const stream = new Duplex();
    stream.push(buffer);
    stream.push(null);
    return stream;
};
