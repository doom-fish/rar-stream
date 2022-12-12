import { parse } from "binary";
export class TerminatorHeaderParser {
  static HEADER_SIZE = 7;
  constructor(headerBuffer) {
    this.headerBuffer = headerBuffer;
  }
  parse() {
    const { vars } = parse(this.headerBuffer)
      .word16lu("crc")
      .word8lu("type")
      .word16lu("flags")
      .word16lu("size");

    return vars;
  }
}
