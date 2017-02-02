// @flow
import binary from 'binary';
import AbstractParser from './abstract-parser';

export default class ArchiveHeaderParser extends AbstractParser {
    static bytesToRead = 13;
    _parseFlags() {
        return function(parsedVars: Object) {
            parsedVars.hasVolumeAttributes = (parsedVars.flags & 0x0001) !== 0;
            parsedVars.hasComment = (parsedVars.flags & 0x0002) !== 0;
            parsedVars.isLocked = (parsedVars.flags & 0x0004) !== 0;
            parsedVars.hasSolidAttributes = (parsedVars.flags & 0x0008) !== 0;
            parsedVars.isNewNameScheme = (parsedVars.flags & 0x00010) !== 0;
            parsedVars.hasAuthInfo = (parsedVars.flags & 0x0020) !== 0;
            parsedVars.hasRecovery = (parsedVars.flags & 0x0040) !== 0;
            parsedVars.isBlockEncoded = (parsedVars.flags & 0x0080) !== 0;
            parsedVars.isFirstVolume = (parsedVars.flags & 0x0100) !== 0;
        };
    }
    get bytesToRead(): number {
        return ArchiveHeaderParser.bytesToRead;
    }
    parse(): Object {
        let {vars: archiveHeader} = binary
            .parse(this.read())
            .word16lu('crc')
            .word8lu('type')
            .word16lu('flags')
            .word16lu('size')
            .word16lu('reserved1')
            .word32lu('reserved2')
            .tap(this._parseFlags());
        archiveHeader.size = archiveHeader.size || this.bytesToRead;
        return archiveHeader;
    }
}
