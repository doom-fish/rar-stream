//@flow
import RarStream from './rar-stream'
import RarFileChunk from './rar-file-chunk';
import streamToBuffer from 'stream-to-buffer';
import type {FileInterval} from '../file-media/file-media';

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
  constructor(name: string, rarFileChunks: RarFileChunk[]){
    this._rarFileChunks = rarFileChunks;
    this._chunkMap = this._calculateChunkMap(rarFileChunks);
    this._size = this._rarFileChunks.reduce((size, chunk) => (size + chunk.length), 0);

    this._name = name;
  }
  readToEnd() : Promise<Buffer> {
    return new Promise((resolve, reject) => {

      streamToBuffer(this.createReadStream({start: 0, end: this._size}), (err, buffer) => {
        if (err) {
          reject(err);
        } else {
          resolve (buffer);
        }
      });
    });
  }
  createReadStream(interval: FileInterval): RarStream {
    if(!interval){
      interval = {start: 0, end: this._size};
    }
    const {start, end} = interval;

    if(start < 0 || end > this._size){
      throw Error('Illegal start/end offset');
    }

    let rarFileChunks = [...this._rarFileChunks];
    rarFileChunks     = this._adjustStartOffset(start, rarFileChunks);
    rarFileChunks     = this._adjustEndOffset(end, rarFileChunks);
    return new RarStream(...rarFileChunks);
  }
  get name () :string{
    return this._name;
  }
  get size () : number {
    return this._size;
  }
  _calculateChunkMap(rarFileChunks: RarFileChunk[]) : ChunkMapping[] {
    const chunkMap = [];

    for(const chunk of rarFileChunks){
      const previousChunk = chunkMap[chunkMap.length -1];
      const start = previousChunk && (previousChunk.end) || 0;
      const end = start + chunk.length;
      chunkMap.push({start, end, chunk});
    }

    return chunkMap;
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
    return rarFileChunks;
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

    return rarFileChunks;
  }
}
