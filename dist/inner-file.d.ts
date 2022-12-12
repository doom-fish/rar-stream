/// <reference types="node" resolution-mode="require"/>
import { IFileMedia, IReadInterval } from "./interfaces.js";
import { InnerFileStream } from "./inner-file-stream.js";
import { RarFileChunk } from "./rar-file-chunk.js";
type ChunkMapEntry = {
    index: number;
    start: number;
    end: number;
    chunk: RarFileChunk;
};
export declare class InnerFile implements IFileMedia {
    name: string;
    private rarFileChunks;
    length: number;
    chunkMap: ChunkMapEntry[];
    constructor(name: string, rarFileChunks: RarFileChunk[]);
    readToEnd(): Promise<Buffer>;
    getChunksToStream(fileStart: number, fileEnd: number): RarFileChunk[];
    createReadStream(interval: IReadInterval): InnerFileStream;
    calculateChunkMap(rarFileChunks: RarFileChunk[]): ChunkMapEntry[];
    findMappedChunk(offset: number): ChunkMapEntry;
}
export {};
//# sourceMappingURL=inner-file.d.ts.map