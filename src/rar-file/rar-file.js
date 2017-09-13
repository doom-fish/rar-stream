const RarStream = require('./rar-stream');
const { streamToBuffer } = require('../stream-utils');

module.exports = class RarFile {
  constructor(name, rarFileChunks) {
    this.rarFileChunks = rarFileChunks;
    this.chunkMap = this.calculateChunkMap(rarFileChunks);
    this.length = this.rarFileChunks.reduce(
      (length, chunk) => length + chunk.length,
      0
    );
    this.name = name;
  }
  readToEnd() {
    return streamToBuffer(
      this.createReadStream({ start: 0, end: this.length })
    );
  }
  getChunksToStream(start, end) {
    const { index: startIndex, start: startOffset } = this.findMappedChunk(
      start
    );

    const { index: endIndex, end: endOffset } = this.findMappedChunk(end);

    const chunksToStream = this.rarFileChunks.slice(startIndex, endIndex + 1);

    const last = chunksToStream.length - 1;
    chunksToStream[0] = chunksToStream[0].paddStart(
      Math.abs(startOffset - start)
    );

    chunksToStream[last] = chunksToStream[last].paddEnd(
      Math.abs(endOffset - end)
    );

    return chunksToStream;
  }
  createReadStream(interval) {
    if (!interval) {
      interval = { start: 0, end: this.length };
    }
    const { start, end } = interval;

    if (start < 0 || end > this.length) {
      throw Error('Illegal start/end offset');
    }

    return new RarStream(this.getChunksToStream(start, end));
  }
  calculateChunkMap(rarFileChunks) {
    const chunkMap = [];
    let index = 0;
    for (const chunk of rarFileChunks) {
      const previousChunk = chunkMap[chunkMap.length - 1];
      const start = (previousChunk && previousChunk.end) || 0;
      const end = start + chunk.length;
      chunkMap.push({ index, start, end, chunk });
      index++;
    }

    return chunkMap;
  }
  findMappedChunk(offset) {
    let selectedMap = this.chunkMap[0];
    for (const map of this.chunkMap) {
      if (offset >= map.start && offset <= map.end) {
        selectedMap = map;
        break;
      }
    }
    return selectedMap;
  }
};
