import { bufferFromString } from "./utils.js";
import { MockFileStream } from "./mock-file-stream.js";
import { IFileMedia, IReadInterval } from "../../interfaces.js";
export class MockFileMedia implements IFileMedia {
  buffer: Buffer;
  constructor(stringData: string, name: string = "MockStream") {
    this.buffer = bufferFromString(stringData.replace(/\s/g, ""));
    const byteLength = stringData.length;
    this.name = name;
    this.length = byteLength / 2;
  }
  length: number;
  name: string;
  createReadStream(options: IReadInterval) {
    return Promise.resolve(
      new MockFileStream(this.buffer, options)
    );
  }
}
