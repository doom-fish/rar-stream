/// <reference types="node" resolution-mode="require"/>
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
    timestamp: number;
    version: number;
    method: number;
    nameSize: number;
    attributes: number;
    name: string;
}
export type IFileHeader = IFileHeaderRaw & IFileHeaderFlags;
export declare class FileHeaderParser {
    private buffer;
    static HEADER_SIZE: number;
    offset: number;
    constructor(buffer: Buffer);
    private handleHighFileSize;
    private parseFileName;
    private parseFlags;
    parse(): IFileHeader;
}
export {};
//# sourceMappingURL=file-header-parser.d.ts.map