const binary = require('binary');

function addSizeIfFlagIsSet(parsedVars) {
    if ((parsedVars.flags & 0x8000) !== 0) {
        let { vars: { addSize } } = this.word32lu('addSize');
        parsedVars.size += addSize || 0;
    }
}
class MarkerHeaderParser {
    constructor(stream) {
        this.stream = stream;
    }

    parse() {
        const markerHeaderBuffer = this.stream.read(
            MarkerHeaderParser.HEADER_SIZE
        );
        const { vars } = binary
            .parse(markerHeaderBuffer)
            .word16lu('crc')
            .word8lu('type')
            .word16lu('flags')
            .word16lu('size')
            .tap(addSizeIfFlagIsSet);

        return vars;
    }
}
MarkerHeaderParser.HEADER_SIZE = 11;
module.exports = MarkerHeaderParser;
