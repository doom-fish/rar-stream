export class TerminatorHeaderParser {
    headerBuffer;
    static HEADER_SIZE = 7;
    constructor(headerBuffer) {
        this.headerBuffer = headerBuffer;
    }
    parse() {
        const crc = this.headerBuffer.readUInt16LE(0);
        const type = this.headerBuffer.readUInt8(2);
        const flags = this.headerBuffer.readUInt16LE(3);
        const size = this.headerBuffer.readUInt16LE(5);
        return { crc, type, flags, size };
    }
}
