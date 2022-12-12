import { bufferFromString } from "./utils";
import { MockFileStream } from "./mock-file-stream";
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
