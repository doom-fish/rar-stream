import binary from 'binary';
import AbstractParser from './abstract-parser';
export default class MarkerHeaderParser extends AbstractParser {
  constructor(stream) {
    super(stream);
  }
  _addSizeIfFlagIsSet(parsedVars) {
    if ((parsedVars.flags & 0x8000) !== 0) {
      let { vars: { addSize } } = this.word32lu('addSize');
      parsedVars.size += addSize || 0;
    }
  }
  get bytesToRead() {
    return 11;
  }
  parse() {
    let { vars: markerHeader } = binary.parse(this.read())
                                    .word16lu('crc')
                                    .word8lu('type')
                                    .word16lu('flags')
                                    .word16lu('size')
                                    .tap(this._addSizeIfFlagIsSet);
    return markerHeader;
  }
}
