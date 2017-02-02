// @flow
import binary from 'binary';
import AbstractParser from './abstract-parser';

export default class TerminatorHeaderParser extends AbstractParser {
    static bytesToRead = 7;
    static endOfArchivePadding = 20;
    get bytesToRead () : number {
        return TerminatorHeaderParser.bytesToRead;
    }
    parse () : Object {
        let { vars: terminatorHeader } = binary.parse(this.read())
                                           .word16lu('crc')
                                           .word8lu('type')
                                           .word16lu('flags')
                                           .word16lu('size');

        return terminatorHeader;
    }
}
