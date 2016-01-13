import {Readable} from "stream";

export default class AbstractParser {
  constructor(stream) {
    if (!(stream instanceof Readable)) {
      throw Error("Invalid Arguments, stream needs to be a ReadableStream instance");
    }
    this._stream = stream;
  }
  get size() {
    throw Error("Abstract Getter, implement in sub classes");
  }
  parse() {
    throw Error("Abstract Method, implement in sub classes");
  }
}
