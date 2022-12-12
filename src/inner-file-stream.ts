import { Readable, ReadableOptions } from "stream";
import { RarFileChunk } from "./rar-file-chunk.js";

export class InnerFileStream extends Readable {
  stream?: NodeJS.ReadableStream;
  constructor(
    private rarFileChunks: RarFileChunk[],
    options?: ReadableOptions
  ) {
    super(options);
  }
  pushData(data: Uint16Array) {
    if (!this.push(data)) {
      this.stream?.pause();
    }
  }
  get isStarted() {
    return !!this.stream;
  }
  next() {
    const chunk = this.rarFileChunks.shift();

    if (!chunk) {
      this.push(null);
    } else {
      this.stream = chunk.getStream();
      this.stream?.on("data", (data) => this.pushData(data));
      this.stream?.on("end", () => this.next());
    }
  }
  override _read() {
    if (!this.isStarted) {
      this.next();
    } else {
      this.stream?.resume();
    }
  }
}
