/// <reference types="node" resolution-mode="require"/>
export declare class ArchiveHeaderParser {
    private buffer;
    static HEADER_SIZE: number;
    constructor(buffer: Buffer);
    parse(): {
        crc: number;
        type: number;
        flags: number;
        size: number;
        reserved1: number;
        reserved2: number;
        hasVolumeAttributes: boolean;
        hasComment: boolean;
        isLocked: boolean;
        hasSolidAttributes: boolean;
        isNewNameScheme: boolean;
        hasAuthInfo: boolean;
        hasRecovery: boolean;
        isBlockEncoded: boolean;
        isFirstVolume: boolean;
    };
}
//# sourceMappingURL=archive-header-parser.d.ts.map