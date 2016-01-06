import binary from "binary";
import AbstractParser from "./abstract-parser";

export default class TerminatorHeaderParser extends AbstractParser {
  constructor(buffer) {
    super(buffer);
  }

  parse() {
    let { vars: terminatorHeader } = binary.parse(this._buffer);

    return terminatorHeader;
  }
}
