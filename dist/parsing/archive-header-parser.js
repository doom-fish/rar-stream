function parseFlags(parsedVars) {
    return {
        hasVolumeAttributes: (parsedVars.flags & 0x0001) !== 0,
        hasComment: (parsedVars.flags & 0x0002) !== 0,
        isLocked: (parsedVars.flags & 0x0004) !== 0,
        hasSolidAttributes: (parsedVars.flags & 0x0008) !== 0,
        isNewNameScheme: (parsedVars.flags & 0x00010) !== 0,
        hasAuthInfo: (parsedVars.flags & 0x0020) !== 0,
        hasRecovery: (parsedVars.flags & 0x0040) !== 0,
        isBlockEncoded: (parsedVars.flags & 0x0080) !== 0,
        isFirstVolume: (parsedVars.flags & 0x0100) !== 0,
    };
}
export class ArchiveHeaderParser {
    buffer;
    static HEADER_SIZE = 13;
    constructor(buffer) {
        this.buffer = buffer;
    }
    parse() {
        const crc = this.buffer.readUInt16LE(0);
        const type = this.buffer.readUInt8(2);
        const flags = this.buffer.readUInt16LE(3);
        let size = this.buffer.readUInt16LE(5);
        const reserved1 = this.buffer.readUInt16LE(7);
        const reserved2 = this.buffer.readUInt32LE(9);
        let vars = { crc, type, flags, size, reserved1, reserved2 };
        return { ...parseFlags(vars), ...vars };
    }
}
