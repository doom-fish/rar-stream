import {Buffer} from 'buffer';
import  binary from 'binary';

import AbstractParser from './AbstractParser';

export default class MarkHeadParser extends AbstractParser {
  constructor(buffer){
    super();
    if(!(buffer instanceof Buffer)){
      throw Error('Invalid Arguments, buffer needs to be a Buffer instance');
    }
    this._buffer = buffer;
  }
  _addSizeIfFlagIsSet(parsedVars){
    if((parsedVars.flags & 0x8000) !== 0){
      let { vars: { add_size } } = this.word32lu("add_size");
      parsedVars.size = parsedVars.size + (add_size || 0);
    }
  }
  parse(){
    let { vars: markerHead } = binary.parse(this._buffer)
                                    .word16lu("crc")
                                    .word8lu("type")
                                    .word16lu("flags")
                                    .word16lu("size")
                                    .tap(this._addSizeIfFlagIsSet);
    return markerHead;
  }
}