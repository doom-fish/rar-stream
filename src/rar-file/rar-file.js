//@flow
import RarStream from './rar-stream'
import RarFileChunk from './rar-file-chunk';
export default class RarFile{
  _rarFileChunks: RarFileChunk[];

  constructor(...rarFileChunks: RarFileChunk[]){
    this._rarFileChunks = rarFileChunks;
  }
  
  createReadStream(startOffset: number, endOffset: number): RarStream {
    this._adjustStartOffset(startOffset);
    this._adjustEndOffset(endOffset);
    return new RarStream(...this._rarFileChunks);
  }
  _adjustStartOffset(startOffset: number): void {
    let startOffsetCopy = startOffset;
    while(startOffset > 0 && this._rarFileChunks.length > 1){
      startOffset -= this._rarFileChunks[0].length;
      this._rarFileChunks.shift();
    }
    this._rarFileChunks[0].startOffset = startOffsetCopy;
  }
  _adjustEndOffset(endOffset :number): void{
    let size = this._rarFileChunks.reduce((size, chunk) => (size + chunk.length), 0);
    while(endOffset <= size && this._rarFileChunks.length > 1){
      size -= this._rarFileChunks[this._rarFileChunks.length - 1].length;
      this._rarFileChunks.pop();
    }
    this._rarFileChunks[this._rarFileChunks.length - 1].endOffset = endOffset;
  }
}
