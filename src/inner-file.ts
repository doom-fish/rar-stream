import { IFileMedia, IReadInterval } from "./interfaces.js";
import { InnerFileStream } from "./inner-file-stream.js";
import { RarFileChunk } from "./rar-file-chunk.js";
import { streamToBuffer } from "./stream-utils.js";
import { sum } from "./utils.js";
type ChunkMapEntry = {
  index: number;
  start: number;
  end: number;
  chunk: RarFileChunk;
};

export class InnerFile implements IFileMedia {
  length: number;

  chunkMap: ChunkMapEntry[];
  constructor(public name: string, private rarFileChunks: RarFileChunk[]) {
    this.length = sum(rarFileChunks.map((c) => c.length));
    this.chunkMap = this.calculateChunkMap(rarFileChunks);

    this.name = name;
  }
  async readToEnd() {
    const stream = await this.createReadStream({ start: 0, end: this.length - 1 });
    return streamToBuffer(stream);
  }
  getChunksToStream(fileStart: number, fileEnd: number) {
    const { index: startIndex, start: startOffset } =
      this.findMappedChunk(fileStart);
    let { index: endIndex, end: endOffset } = this.findMappedChunk(fileEnd);

    const chunksToStream = this.rarFileChunks.slice(startIndex, endIndex + 1);

    const last = chunksToStream.length - 1;
    const first = 0;
    chunksToStream[first] = chunksToStream[first]!.padStart(
      Math.abs(startOffset - fileStart)
    );

    let diff = Math.abs(endOffset - fileEnd);
    if (diff === this.rarFileChunks.length) {
      diff = 0;
    }
    if (diff !== 0) {
      chunksToStream[last] = chunksToStream[last]!.padEnd(diff);
    }

    return chunksToStream;
  }
  createReadStream(interval: IReadInterval) {
    if (!interval) {
      interval = { start: 0, end: this.length - 1 };
    }
    let { start, end } = interval;

    if (start < 0 || end >= this.length) {
      throw Error("Illegal start/end offset");
    }

    return Promise.resolve(
      new InnerFileStream(this.getChunksToStream(start, end))
    );
  }
  calculateChunkMap(rarFileChunks: RarFileChunk[]) {
    const chunkMap: ChunkMapEntry[] = [];
    let index = 0;
    let fileOffset = 0;
    for (const chunk of rarFileChunks) {
      const start = fileOffset;
      const end = fileOffset + chunk.length;
      fileOffset = end + 1;

      chunkMap.push({ index, start, end, chunk });
      index++;
    }

    return chunkMap;
  }
  findMappedChunk(offset: number) {
    let selectedMap = this.chunkMap[0]!;
    for (const chunkMapping of this.chunkMap) {
      if (offset >= chunkMapping.start && offset <= chunkMapping.end) {
        selectedMap = chunkMapping;
        break;
      }
    }
    return selectedMap;
  }
}
