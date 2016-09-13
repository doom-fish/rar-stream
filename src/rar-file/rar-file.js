//@flow
import RarStream from './rar-stream'
import RarFileChunk from './rar-file-chunk';
export default class RarFile{
  _RarFileChunks: RarFileChunk[];

  constructor(RarFileChunks: RarFileChunk[]){
    this._RarFileChunks = RarFileChunks;
  }
  get size () : number {
      return this._RarFileChunks.reduce((size, chunk) => (size + chunk.length), 0);
  }
  createReadStream(startOffset: number, endOffset: number): RarStream {
    this._adjustStartOffset(startOffset);
    this._adjustEndOffset(endOffset);
    return new RarStream(this._RarFileChunks);
  }
  _adjustStartOffset(startOffset: number): void {
    let startOffsetCopy = startOffset;
    while(startOffset > 0 && this._RarFileChunks.length > 1){
      startOffset -= this._RarFileChunks[0].length;
      this._RarFileChunks.shift();
    }
    this._RarFileChunks[0].startOffset = startOffsetCopy;
  }
  _adjustEndOffset(endOffset :number): void{
    let size = this.size;
    while(endOffset <= size && this._RarFileChunks.length > 1){
      size -= this._RarFileChunks[this._RarFileChunks.length - 1].length;
      this._RarFileChunks.pop();
    }
    this._RarFileChunks[this._RarFileChunks.length - 1].endOffset = endOffset;
  }
}
