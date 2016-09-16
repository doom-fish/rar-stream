//@flow
import {Readable} from 'stream';
import binary from 'binary';
import AbstractParser from './abstract-parser';

export default class MarkerHeaderParser extends AbstractParser {
  static bytesToRead = 11;
  constructor(stream: Readable) {
    super(stream);
  }
  _addSizeIfFlagIsSet() {
    return function (parsedVars: Object) {
        if ((parsedVars.flags & 0x8000) !== 0) {
          let { vars: { addSize } } = this.word32lu('addSize');
          parsedVars.size += addSize || 0;
        }
    };
  }
  get bytesToRead():number {
    return MarkerHeaderParser.bytesToRead;
  }
  parse() : Object {
    let { vars: markerHeader } = binary.parse(this.read())
                                    .word16lu('crc')
                                    .word8lu('type')
                                    .word16lu('flags')
                                    .word16lu('size')
                                    .tap(this._addSizeIfFlagIsSet());

    return markerHeader;
  }
}
