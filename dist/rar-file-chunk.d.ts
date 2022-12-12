/// <reference types="node" resolution-mode="require"/>
import { IFileMedia } from "./interfaces.js";
export declare class RarFileChunk {
    private fileMedia;
    startOffset: number;
    endOffset: number;
    constructor(fileMedia: IFileMedia, startOffset: number, endOffset: number);
    padEnd(endPadding: number): RarFileChunk;
    padStart(startPadding: number): RarFileChunk;
    get length(): number;
    getStream(): NodeJS.ReadableStream;
}
//# sourceMappingURL=rar-file-chunk.d.ts.map