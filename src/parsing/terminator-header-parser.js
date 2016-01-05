import {Buffer} from "buffer";
import binary from "binary";

import AbstractParser from "./abstract-parser";

export default class TerminatorHeaderParser extends AbstractParser {
  constructor(buffer) {
    super();
    if (!(buffer instanceof Buffer)) {
      throw Error("Invalid Arguments, buffer needs to be a Buffer instance");
    }
    this._buffer = buffer;
  }

  parse() {
    let { vars: terminatorHeader } = binary.parse(this._buffer);

    return terminatorHeader;
  }
}
