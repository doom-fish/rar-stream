import { Stream } from "stream";
import { bufferFromString } from "./utils";
import { MockFileStream } from "./mock-file-stream";
import { IFileMedia, IReadInterval } from "../../interfaces";
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
    return new MockFileStream(this.buffer, options);
  }
}
