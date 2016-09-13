//@flow
import RarStream from './rar-stream'
import FileChunk from './file-chunk';
export default class RarFile{
  _fileChunks: FileChunk[];

  constructor(fileChunks: FileChunk[]){
    this._fileChunks = fileChunks;
  }
  get size () : number {
      return this._fileChunks.reduce((size, chunk) => (size + chunk.length), 0);
  }
  createReadStream(startOffset: number, endOffset: number): RarStream {
    this._adjustStartOffset(startOffset);
    this._adjustEndOffset(endOffset);
    return new RarStream(this._fileChunks);
  }
  _adjustStartOffset(startOffset: number): void {
    let startOffsetCopy = startOffset;
    while(startOffset > 0 && this._fileChunks.length > 1){
      startOffset -= this._fileChunks[0].length;
      this._fileChunks.shift();
    }
    this._fileChunks[0].startOffset = startOffsetCopy;
  }
  _adjustEndOffset(endOffset :number): void{
    let size = this.size;
    while(endOffset <= size && this._fileChunks.length > 1){
      size -= this._fileChunks[this._fileChunks.length - 1].length;
      this._fileChunks.pop();
    }
    this._fileChunks[this._fileChunks.length - 1].endOffset = endOffset;
  }
}
