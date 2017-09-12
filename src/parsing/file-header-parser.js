const binary = require('binary');

function parseFlags(parsedVars) {
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
}
function parseFileName(parsedVars) {
    let { vars } = this.buffer('nameBuffer', parsedVars.nameSize);
    parsedVars.name = vars.nameBuffer.toString('utf-8');
}
function handleHighFileSize(parsedVars) {
    if (parsedVars.hasHighSize) {
        let { vars } = this.word32ls('highPackSize').word32ls('highUnpackSize');
        const { highPackSize, highUnpackSize } = vars;
        parsedVars.size = highPackSize * 0x100000000 + parsedVars.size;
        parsedVars.unpackedSize =
            highUnpackSize * 0x100000000 + parsedVars.unpackedSize;
    }
}

class FileHeaderParser {
    constructor(stream) {
        this.stream = stream;
    }
    parse() {
        const fileHeaderBuffer = this.stream.read(FileHeaderParser.HEADER_SIZE);
        const { vars } = binary
            .parse(fileHeaderBuffer)
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
            .tap(parseFlags)
            .tap(handleHighFileSize)
            .tap(parseFileName);
        return vars;
    }
}

FileHeaderParser.HEADER_SIZE = 280;
module.exports = FileHeaderParser;
