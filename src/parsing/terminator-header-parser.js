// @flow
import binary from 'binary';
import AbstractParser from './abstract-parser';

type TerminatorHeader = {
    crc: number,
    type: number,
    flags: number,
    size: number
};

export default class TerminatorHeaderParser extends AbstractParser {
    static bytesToRead = 7;
    static endOfArchivePadding = 20;
    get bytesToRead(): number {
        return TerminatorHeaderParser.bytesToRead;
    }
    parse(): TerminatorHeader {
        let { vars: terminatorHeader } = binary
            .parse(this.read())
            .word16lu('crc')
            .word8lu('type')
            .word16lu('flags')
            .word16lu('size');

        return terminatorHeader;
    }
}
