import binary from "binary";
import AbstractParser from "./abstract-parser";

export default class TerminatorHeaderParser extends AbstractParser {
  constructor(stream) {
    super(stream);
  }
  get size() {
    return 7;
  }
  parse() {
    let buffer = this._stream.read(this.size);
    let { vars: terminatorHeader } = binary.parse(buffer)
                                           .word16lu("crc")
                                           .word8lu("type")
                                           .word16lu("flags")
                                           .word16lu("size");

    return terminatorHeader;
  }
}
