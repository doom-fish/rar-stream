interface IFileHeaderFlags {
  continuesFromPrevious: boolean;
  continuesInNext: boolean;
  isEncrypted: boolean;
  hasComment: boolean;
  hasInfoFromPrevious: boolean;
  hasHighSize: boolean;
  hasSpecialName: boolean;
  hasSalt: boolean;
  isOldVersion: boolean;
  hasExtendedTime: boolean;
}

interface IFileHeaderRaw {
  crc: number;
  type: number;
  flags: number;
  headSize: number;
  size: number;
  unpackedSize: number;
  host: number;
  fileCrc: number;
  timestamp: number | null;
  version: number | null;
  method: number | null;
  nameSize: number;
  attributes: number | null;
  name: string;
}

export type IFileHeader = IFileHeaderRaw & IFileHeaderFlags;
export class FileHeaderParser {
  static HEADER_SIZE = 280;
  offset = 0;
  constructor(private buffer: Buffer) {}
  private handleHighFileSize(parsedVars: IFileHeader) {
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
  private parseFileName(parsedVars: IFileHeaderRaw) {

    parsedVars.name = this.buffer
      .subarray(this.offset, this.offset + parsedVars.nameSize)
      .toString("utf-8");
  }
  private parseFlags(parsedVars: IFileHeaderRaw): IFileHeaderFlags {
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
  parse(): IFileHeader {
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

    let timestamp = null;
    try {
        timestamp = this.buffer.readUInt32LE(this.offset);
    } catch(e) {}
    this.offset += 4;

    let version = null;
    try {
        version = this.buffer.readUInt8(this.offset);
    } catch(e) {}
    this.offset += 1;

    let method = null;
    try {
        method = this.buffer.readUInt8(this.offset);
    } catch(e) {}
    this.offset += 1;

    let nameSize = 0;
    try {
        nameSize = this.buffer.readUInt16LE(this.offset);
    } catch(e) {}
    this.offset += 2;

    let attributes = null;
    try {
        attributes = this.buffer.readUInt32LE(this.offset);
    } catch(e) {}
    this.offset += 4;

    let vars: IFileHeaderRaw = {
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
