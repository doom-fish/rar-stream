import { parse } from "binary";

function addSizeIfFlagIsSet(parsedVars) {
  if ((parsedVars.flags & 0x8000) !== 0) {
    let {
      vars: { addSize },
    } = this.word32lu("addSize");
    parsedVars.size += addSize || 0;
  }
}
export class MarkerHeaderParser {
  static HEADER_SIZE = 11;
  constructor(headerBuffer) {
    this.headerBuffer = headerBuffer;
  }

  parse() {
    const { vars } = parse(this.headerBuffer)
      .word16lu("crc")
      .word8lu("type")
      .word16lu("flags")
      .word16lu("size")
      .tap(addSizeIfFlagIsSet);

    return vars;
  }
}
