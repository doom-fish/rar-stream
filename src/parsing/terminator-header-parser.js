const binary = require('binary');
class TerminatorHeaderParser {
    constructor(headerBuffer) {
        this.headerBuffer = headerBuffer;
    }
    parse() {
        const { vars } = binary
            .parse(this.headerBuffer)
            .word16lu('crc')
            .word8lu('type')
            .word16lu('flags')
            .word16lu('size');

        return vars;
    }
}

TerminatorHeaderParser.HEADER_SIZE = 7;
TerminatorHeaderParser.END_OF_ARCHIVE_PADDING = 20;
module.exports = TerminatorHeaderParser;
