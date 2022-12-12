import { InnerFileStream } from "./inner-file-stream.js";
import { streamToBuffer } from "./stream-utils.js";

export class InnerFile {
  constructor(name, rarFileChunks) {
    this.rarFileChunks = rarFileChunks;
    this.length = this.rarFileChunks.reduce(
      (length, chunk) => length + chunk.length,
      0
    );
    this.chunkMap = this.calculateChunkMap(rarFileChunks);

    this.name = name;
  }
  readToEnd() {
    return streamToBuffer(
      this.createReadStream({ start: 0, end: this.length - 1 })
    );
  }
  getChunksToStream(fileStart, fileEnd) {
    const { index: startIndex, start: startOffset } =
      this.findMappedChunk(fileStart);
    let { index: endIndex, end: endOffset } = this.findMappedChunk(fileEnd);

    const chunksToStream = this.rarFileChunks.slice(startIndex, endIndex + 1);

    const last = chunksToStream.length - 1;
    const first = 0;
    chunksToStream[first] = chunksToStream[first].padStart(
      Math.abs(startOffset - fileStart)
    );

    let diff = Math.abs(endOffset - fileEnd);
    if (diff === this.rarFileChunks.length) {
      diff = 0;
    }
    if (diff !== 0) {
      chunksToStream[last] = chunksToStream[last].padEnd(diff);
    }

    return chunksToStream;
  }
  createReadStream(interval) {
    if (!interval) {
      interval = { start: 0, end: this.length - 1 };
    }
    let { start, end } = interval;

    if (start < 0 || end >= this.length) {
      throw Error("Illegal start/end offset");
    }

    return new InnerFileStream(this.getChunksToStream(start, end));
  }
  calculateChunkMap(rarFileChunks) {
    const chunkMap = [];
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
  findMappedChunk(offset) {
    let selectedMap = this.chunkMap[0];
    for (const chunkMapping of this.chunkMap) {
      if (offset >= chunkMapping.start && offset <= chunkMapping.end) {
        selectedMap = chunkMapping;
        break;
      }
    }
    return selectedMap;
  }
}
