export class FileHeaderParser {
    buffer;
    static HEADER_SIZE = 280;
    offset = 0;
    constructor(buffer) {
        this.buffer = buffer;
    }
    handleHighFileSize(parsedVars) {
        if (parsedVars.hasHighSize) {
            const highPackSize = this.buffer.readInt32LE(this.offset);
            this.offset += 4;
            const highUnpackSize = this.buffer.readInt32LE(this.offset);
            this.offset += 4;
            parsedVars.size = highPackSize * 0x100000000 + parsedVars.size;
            parsedVars.unpackedSize =
                highUnpackSize * 0x100000000 + parsedVars.unpackedSize;
        }
    }
    parseFileName(parsedVars) {
        parsedVars.name = this.buffer
            .subarray(this.offset, this.offset + parsedVars.nameSize)
            .toString("utf-8");
    }
    parseFlags(parsedVars) {
        return {
            continuesFromPrevious: (parsedVars.flags & 0x01) !== 0,
            continuesInNext: (parsedVars.flags & 0x02) !== 0,
            isEncrypted: (parsedVars.flags & 0x04) !== 0,
            hasComment: (parsedVars.flags & 0x08) !== 0,
            hasInfoFromPrevious: (parsedVars.flags & 0x10) !== 0,
            hasHighSize: (parsedVars.flags & 0x100) !== 0,
            hasSpecialName: (parsedVars.flags & 0x200) !== 0,
            hasSalt: (parsedVars.flags & 0x400) !== 0,
            isOldVersion: (parsedVars.flags & 0x800) !== 0,
            hasExtendedTime: (parsedVars.flags & 0x1000) !== 0,
        };
    }
    parse() {
        const crc = this.buffer.readUInt16LE(this.offset);
        this.offset += 2;
        const type = this.buffer.readUInt8(this.offset);
        this.offset += 1;
        const flags = this.buffer.readUInt16LE(this.offset);
        this.offset += 2;
        const headSize = this.buffer.readUInt16LE(this.offset);
        this.offset += 2;
        const size = this.buffer.readUInt32LE(this.offset);
        this.offset += 4;
        const unpackedSize = this.buffer.readUInt32LE(this.offset);
        this.offset += 4;
        const host = this.buffer.readUInt8(this.offset);
        this.offset += 1;
        const fileCrc = this.buffer.readUInt32LE(this.offset);
        this.offset += 4;
        const timestamp = this.buffer.readUInt32LE(this.offset);
        this.offset += 4;
        const version = this.buffer.readUInt8(this.offset);
        this.offset += 1;
        const method = this.buffer.readUInt8(this.offset);
        this.offset += 1;
        const nameSize = this.buffer.readUInt16LE(this.offset);
        this.offset += 2;
        const attributes = this.buffer.readUInt32LE(this.offset);
        this.offset += 4;
        let vars = {
            crc,
            type,
            flags,
            headSize,
            size,
            unpackedSize,
            host,
            fileCrc,
            timestamp,
            version,
            method,
            nameSize,
            attributes,
            name: "",
        };
        const boolFlags = this.parseFlags(vars);
        const header = { ...vars, ...boolFlags };
        this.handleHighFileSize(header);
        this.parseFileName(header);
        this.offset = 0;
        return header;
    }
}
