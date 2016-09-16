//@flow
import RarStream from './rar-stream'
import RarFileChunk from './rar-file-chunk';
import streamToBuffer from 'stream-to-buffer';

export default class RarFile{
  _rarFileChunks: RarFileChunk[];
  _name: string;
  _chunkMap: Object;
  _size: number;
  constructor(name: string, ...rarFileChunks: RarFileChunk[]){
    this._rarFileChunks = rarFileChunks;
    this._chunkMap = [];

    for(const chunk of rarFileChunks){
      const previousChunk = this._chunkMap[this._chunkMap.length -1];
      const start = previousChunk && previousChunk.end || 0;
      const end = start + chunk.length;
      this._chunkMap.push({start, end, chunk});
    }

    this._size = this._rarFileChunks.reduce((size, chunk) => (size + chunk.length), 0);
    this._name = name;
  }
  readToEnd() : Promise<Buffer> {
    return new Promise((resolve, reject) => {

      streamToBuffer(this.createReadStream(0, this.size), (err, buffer) => {
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
  _adjustStartOffset(startOffset: number, rarFileChunks: RarFileChunk[]): RarFileChunk[] {
    let selectedMap;
    for(const map of this._chunkMap){
      if(startOffset >= map.start && startOffset <= map.end){
        selectedMap = map;
        break;
      }
    }
    while(rarFileChunks[0] !== selectedMap.chunk) {
      rarFileChunks.shift();
    }
    selectedMap.chunk._startOffset += Math.abs(startOffset - selectedMap.start);
    if(rarFileChunks[0]._startOffset === rarFileChunks[0]._endOffset){
      rarFileChunks.shift();
    }
    return rarFileChunks;
  }
  _adjustEndOffset(endOffset :number, rarFileChunks: RarFileChunk[]): RarFileChunk[]{
    let selectedMap;
    for(const map of this._chunkMap){
      if(endOffset >= map.start && endOffset <= map.end){
        selectedMap = map;
        break;
      }
    }
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
    return rarFileChunks;
  }
}
