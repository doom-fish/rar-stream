const binary = require('binary');

function parseFlags(parsedVars) {
    parsedVars.hasVolumeAttributes = (parsedVars.flags & 0x0001) !== 0;
    parsedVars.hasComment = (parsedVars.flags & 0x0002) !== 0;
    parsedVars.isLocked = (parsedVars.flags & 0x0004) !== 0;
    parsedVars.hasSolidAttributes = (parsedVars.flags & 0x0008) !== 0;
    parsedVars.isNewNameScheme = (parsedVars.flags & 0x00010) !== 0;
    parsedVars.hasAuthInfo = (parsedVars.flags & 0x0020) !== 0;
    parsedVars.hasRecovery = (parsedVars.flags & 0x0040) !== 0;
    parsedVars.isBlockEncoded = (parsedVars.flags & 0x0080) !== 0;
    parsedVars.isFirstVolume = (parsedVars.flags & 0x0100) !== 0;
}
class ArchiveHeaderParser {
    constructor(headerBuffer) {
        this.headerBuffer = headerBuffer;
    }
    parse() {
        const { vars } = binary
            .parse(this.headerBuffer)
            .word16lu('crc')
            .word8lu('type')
            .word16lu('flags')
            .word16lu('size')
            .word16lu('reserved1')
            .word32lu('reserved2')
            .tap(parseFlags);
        if (!vars.size) {
            vars.size = ArchiveHeaderParser.HEADER_SIZE;
        }
        return vars;
    }
}
ArchiveHeaderParser.HEADER_SIZE = 13;
module.exports = ArchiveHeaderParser;
