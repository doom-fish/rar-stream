export class MarkerHeaderParser {
    headerBuffer;
    static HEADER_SIZE = 11;
    constructor(headerBuffer) {
        this.headerBuffer = headerBuffer;
    }
    parse() {
        const crc = this.headerBuffer.readUInt16LE(0);
        const type = this.headerBuffer.readUInt8(2);
        const flags = this.headerBuffer.readUInt16LE(3);
        let size = this.headerBuffer.readUInt16LE(5);
        if ((flags & 0x8000) !== 0) {
            let addSize = this.headerBuffer.readUint32LE(7);
            size += addSize || 0;
        }
        return { crc, type, flags, size };
    }
}
