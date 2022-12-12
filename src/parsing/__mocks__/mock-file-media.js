import { bufferFromString } from "./utils.js";
import { MockFileStream } from "./mock-file-stream.js";
export class MockFileMedia {
  constructor(stringData, name = "MockStream") {
    this.buffer = bufferFromString(stringData.replace(/\s/g, ""));
    const byteLength = stringData.length;
    this.name = name;
    this.length = byteLength / 2;
  }
  createReadStream(options) {
    return new MockFileStream(this.buffer, options);
  }
}
