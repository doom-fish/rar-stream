import binary from "binary";
import AbstractParser from "./abstract-parser";

export default class TerminatorHeaderParser extends AbstractParser {
  constructor(stream) {
    super(stream);
  }
  get bytesToRead() {
    return 7;
  }
  parse() {
    let { vars: terminatorHeader } = binary.parse(this.read())
                                           .word16lu("crc")
                                           .word8lu("type")
                                           .word16lu("flags")
                                           .word16lu("size");

    return terminatorHeader;
  }
}
