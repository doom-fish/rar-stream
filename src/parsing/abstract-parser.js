//@flow
import {Readable} from 'stream';

export default class AbstractParser {
  _stream: Readable;
  constructor(stream: Readable) {
    this._stream = stream;
  }
  get bytesToRead(): number {
    throw Error('Abstract Getter, implement in sub classes');
  }
  parse() : Object {
    throw Error('Abstract Method, implement in sub classes');
  }
  read(): ?string | ?Buffer {
    if (!this.bytesToRead || Number.isNaN(this.bytesToRead) || this.bytesToRead < 0) {
      throw Error('Invalid Size, size need to be a positive number');
    }
    return this._stream.read(this.bytesToRead);
  }
}
