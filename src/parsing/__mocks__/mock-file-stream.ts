import { Readable, ReadableOptions } from "stream";
import { IReadInterval } from "../../interfaces.js";

export class MockFileStream extends Readable {
  constructor(private object: Buffer | null, private options: IReadInterval) {
    super(options as ReadableOptions);
  }
  override _read() {
    if (
      !!this.object &&
      typeof this.options.start === "number" &&
      typeof this.options.end === "number"
    ) {
      const buffer = this.object.slice(this.options.start, this.options.end);
      this.push(buffer);
      this.object = null;
    } else {
      this.push(this.object);
      this.object = null;
    }
  }
}
