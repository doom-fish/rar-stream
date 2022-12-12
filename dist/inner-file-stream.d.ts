/// <reference types="node" resolution-mode="require"/>
/// <reference types="node" resolution-mode="require"/>
import { Readable, ReadableOptions } from "stream";
import { RarFileChunk } from "./rar-file-chunk.js";
export declare class InnerFileStream extends Readable {
    private rarFileChunks;
    stream?: NodeJS.ReadableStream;
    constructor(rarFileChunks: RarFileChunk[], options?: ReadableOptions);
    pushData(data: Uint16Array): void;
    get isStarted(): boolean;
    next(): void;
    _read(): void;
}
//# sourceMappingURL=inner-file-stream.d.ts.map