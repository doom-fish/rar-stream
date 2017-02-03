// @flow
import RarStream from './rar-stream';
import RarFileChunk from './rar-file-chunk';
import streamToBuffer from 'stream-to-buffer';
import type { FileInterval } from '../file-media/file-media';

type ChunkMapping = {
    start: number,
    end: number,
    index: number,
    chunk: RarFileChunk
};

export default class RarFile {
    _rarFileChunks: RarFileChunk[];
    _name: string;
    _chunkMap: ChunkMapping[];
    _size: number;
    constructor(name: string, rarFileChunks: RarFileChunk[]) {
        this._rarFileChunks = rarFileChunks;
        this._chunkMap = this._calculateChunkMap(rarFileChunks);
        this._size = this._rarFileChunks.reduce(
            (size, chunk) => size + chunk.length,
            0
        );
        this._name = name;
    }
    readToEnd(): Promise<Buffer> {
        return new Promise((resolve, reject) => {
            streamToBuffer(
                this.createReadStream({ start: 0, end: this._size }),
                (err, buffer) => {
                    if (err) {
                        reject(err);
                    } else {
                        resolve(buffer);
                    }
                }
            );
        });
    }
    getChunksToStream(start: number, end: number): RarFileChunk[] {
        const {
            index: startIndex,
            start: startOffset
        } = this._findMappedChunk(start);

        const {
            index: endIndex,
            end: endOffset
        } = this._findMappedChunk(end);

        const chunksToStream = this._rarFileChunks.slice(
            startIndex,
            endIndex + 1
        );

        const last = chunksToStream.length - 1;
        chunksToStream[0] = chunksToStream[0].paddStart(
            Math.abs(startOffset - start)
        );

        chunksToStream[last] = chunksToStream[last].paddEnd(
            Math.abs(endOffset - end)
        );

        return chunksToStream;
    }
    createReadStream(interval: FileInterval): RarStream {
        if (!interval) {
            interval = { start: 0, end: this._size };
        }
        const { start, end } = interval;

        if (start < 0 || end > this._size) {
            throw Error('Illegal start/end offset');
        }

        return new RarStream(this.getChunksToStream(start, end));
    }
    get name(): string {
        return this._name;
    }
    get size(): number {
        return this._size;
    }
    _calculateChunkMap(rarFileChunks: RarFileChunk[]): ChunkMapping[] {
        const chunkMap = [];
        let index = 0;
        for (const chunk of rarFileChunks) {
            const previousChunk = chunkMap[chunkMap.length - 1];
            const start = previousChunk && previousChunk.end || 0;
            const end = start + chunk.length;
            chunkMap.push({ index, start, end, chunk });
            index++;
        }

        return chunkMap;
    }
    _findMappedChunk(offset: number): ChunkMapping {
        let selectedMap = this._chunkMap[0];
        for (const map of this._chunkMap) {
            if (offset >= map.start && offset <= map.end) {
                selectedMap = map;
                break;
            }
        }
        return selectedMap;
    }
}
