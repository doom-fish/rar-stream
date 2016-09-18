//@flow
import RarStream from './rar-stream'
import RarFileChunk from './rar-file-chunk';
import streamToBuffer from 'stream-to-buffer';

type ChunkMapping = {
  start: number,
  end: number,
  chunk: RarFileChunk
}

export default class RarFile{
  _rarFileChunks: RarFileChunk[];
  _name: string;
  _chunkMap: ChunkMapping[];
  _size: number;
  constructor(name: string, ...rarFileChunks: RarFileChunk[]){
    this._rarFileChunks = rarFileChunks;
    this._chunkMap = this._calculateChunkMap(rarFileChunks);
    this._size = this._rarFileChunks.reduce((size, chunk) => (size + chunk.length), 0);

    this._name = name;
  }
  readToEnd() : Promise<Buffer> {
    return new Promise((resolve, reject) => {

      streamToBuffer(this.createReadStream(0, this.size - 1), (err, buffer) => {
        if (err) {
          reject(err);
        } else {
          resolve (buffer);
        }
      });
    });
  }
  createReadStream(startOffset: number, endOffset: number): RarStream {
    if(startOffset < 0 || endOffset > this._size){
      throw Error('Illegal start/end offset');
    }

    let rarFileChunks = [...this._rarFileChunks];
    rarFileChunks     = this._adjustStartOffset(startOffset, rarFileChunks);
    rarFileChunks     = this._adjustEndOffset(endOffset, rarFileChunks);
    return new RarStream(...rarFileChunks);
  }
  get name () :string{
    return this._name;
  }
  get size () : number {
    return this._size;
  }
  _findMappedChunk(offset: number): ChunkMapping {
    let selectedMap = this._chunkMap[0];
    for(const map of this._chunkMap){
      if(offset >= map.start && offset <= map.end){
        selectedMap = map;
        break;
      }
    }
    return selectedMap;
  }
  _adjustStartOffset(startOffset: number, rarFileChunks: RarFileChunk[]): RarFileChunk[] {
    const selectedMap = this._findMappedChunk(startOffset);

    while(rarFileChunks[0] !== selectedMap.chunk) {
      rarFileChunks.shift();
    }
    selectedMap.chunk._startOffset += Math.abs(startOffset - selectedMap.start);

    if(rarFileChunks[0]._startOffset === rarFileChunks[0]._endOffset){
      rarFileChunks.shift();
    }

    return rarFileChunks;
  }
  _calculateChunkMap(rarFileChunks: RarFileChunk[]) : ChunkMapping[] {
    const chunkMap = [];

    for(const chunk of rarFileChunks){
      const previousChunk = chunkMap[chunkMap.length -1];
      const start = previousChunk && previousChunk.end || 0;
      const end = start + chunk.length;
      chunkMap.push({start, end, chunk});
    }
    return chunkMap;
  }
  _adjustEndOffset(endOffset :number, rarFileChunks: RarFileChunk[]): RarFileChunk[]{
    const selectedMap = this._findMappedChunk(endOffset);

    for(let index = rarFileChunks.length - 1; index >= 0; --index){
      if(rarFileChunks[index] !== selectedMap.chunk){
        rarFileChunks.pop();
      } else {
        break;
      }
    }

    selectedMap.chunk._endOffset -= Math.abs(endOffset - selectedMap.end);

    const lastChunk = rarFileChunks[rarFileChunks.length -1];
    if(lastChunk._startOffset === lastChunk._endOffset){
      rarFileChunks.pop();
    }
    rarFileChunks[rarFileChunks.length -1]._endOffset++;
    return rarFileChunks;
  }
}
