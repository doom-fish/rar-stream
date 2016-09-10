//@flow
import {Readable} from 'stream';
import binary from 'binary';
import AbstractParser from './abstract-parser';

export default class FileHeaderParser extends AbstractParser {
  constructor(stream: Readable) {
    super(stream);
  }
  _parseFlags () {
    return function (parsedVars: Object) {
        parsedVars.continuesFromPrevious = (parsedVars.flags & 0x01) !== 0;
        parsedVars.continuesInNext = (parsedVars.flags & 0x02) !== 0;
        parsedVars.isEncrypted = (parsedVars.flags & 0x04) !== 0;
        parsedVars.hasComment = (parsedVars.flags & 0x08) !== 0;
        parsedVars.hasInfoFromPrevious = (parsedVars.flags & 0x10) !== 0;
        parsedVars.hasHighSize = (parsedVars.flags & 0x100) !== 0;
        parsedVars.hasSpecialName = (parsedVars.flags & 0x200) !== 0;
        parsedVars.hasSalt = (parsedVars.flags & 0x400) !== 0;
        parsedVars.isOldVersion = (parsedVars.flags & 0x800) !== 0;
        parsedVars.hasExtendedTime = (parsedVars.flags & 0x1000) !== 0;
    };
  }
  _parseFileName () {
    return function(parsedVars: Object) {
        let {vars: {nameBuffer}} = this.buffer('nameBuffer', parsedVars.nameSize);
        parsedVars.name = nameBuffer.toString('utf-8');
      };
  }
  _handleHighFileSize () {
    return function (parsedVars: Object) {
        if (parsedVars.hasHighSize) {
          let {vars: {highPackSize, highUnpackSize}} = this.word32ls('highPackSize')
                                                            .word32ls('highUnpackSize');

          parsedVars.size = highPackSize * 0x100000000 + parsedVars.size;
          parsedVars.unpackedSize = highUnpackSize * 0x100000000 + parsedVars.unpackedSize;
        }
    };
  }

  get bytesToRead() : number{
    return 280;
  }
  parse() : Object{
    let {vars: fileHeader} = binary.parse(this.read())
                                   .word16lu('crc')
                                   .word8lu('type')
                                   .word16lu('flags')
                                   .word16lu('headSize')
                                   .word32lu('size')
                                   .word32lu('unpackedSize')
                                   .word8lu('host')
                                   .word32lu('fileCrc')
                                   .word32lu('timestamp')
                                   .word8lu('version')
                                   .word8lu('method')
                                   .word16lu('nameSize')
                                   .word32lu('attributes')
                                   .tap(this._parseFlags())
                                   .tap(this._handleHighFileSize())
                                   .tap(this._parseFileName());
    return fileHeader;
  }
}
