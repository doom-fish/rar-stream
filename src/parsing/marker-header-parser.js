// @flow
import binary from 'binary';
import AbstractParser from './abstract-parser';

type MarkerHeader = {
    crc: number,
    type: number,
    flags: number,
    size: number
};

type Parser = (parsedVars: MarkerHeader) => void;

export default class MarkerHeaderParser extends AbstractParser {
    static bytesToRead = 11;
    _addSizeIfFlagIsSet(): Parser {
        return function(parsedVars: MarkerHeader) {
            if ((parsedVars.flags & 0x8000) !== 0) {
                let { vars: { addSize } } = this.word32lu('addSize');
                parsedVars.size += addSize || 0;
            }
        };
    }
    get bytesToRead(): number {
        return MarkerHeaderParser.bytesToRead;
    }
    parse(): MarkerHeader {
        let { vars: markerHeader } = binary
            .parse(this.read())
            .word16lu('crc')
            .word8lu('type')
            .word16lu('flags')
            .word16lu('size')
            .tap(this._addSizeIfFlagIsSet());

        return markerHeader;
    }
}
