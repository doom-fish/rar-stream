/// <reference types="node" resolution-mode="require"/>
export declare class TerminatorHeaderParser {
    private headerBuffer;
    static HEADER_SIZE: number;
    constructor(headerBuffer: Buffer);
    parse(): {
        crc: number;
        type: number;
        flags: number;
        size: number;
    };
}
//# sourceMappingURL=terminator-header-parser.d.ts.map