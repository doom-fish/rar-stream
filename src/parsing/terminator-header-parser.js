const binary = require('binary');
class TerminatorHeaderParser {
    constructor(stream) {
        this.stream = stream;
    }
    parse() {
        const terminatorHeaderBuffer = this.stream.read(
            TerminatorHeaderParser.HEADER_SIZE
        );
        const { vars } = binary
            .parse(terminatorHeaderBuffer)
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
