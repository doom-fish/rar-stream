//@flow
import binary from 'binary';

import {Readable} from 'stream';
import AbstractParser from './abstract-parser';

export default class TerminatorHeaderParser extends AbstractParser {
  static bytesToRead = 7;
  constructor(stream: Readable) {
    super(stream);
  }
  get bytesToRead() : number{
    return TerminatorHeaderParser.bytesToRead;
  }
  parse() : Object {
    let { vars: terminatorHeader } = binary.parse(this.read())
                                           .word16lu('crc')
                                           .word8lu('type')
                                           .word16lu('flags')
                                           .word16lu('size');

    return terminatorHeader;
  }
}
