//@flow
import {Readable} from 'stream';
import RarFileChunk from './rar-file-chunk';
export default class RarStream extends Readable {
  _rarFileChunks: RarFileChunk[];
  _byteOffset: number = 0;
  _currentChunk: RarFileChunk;
  constructor(...rarFileChunks: RarFileChunk[]){
    super();
    this._rarFileChunks = rarFileChunks;
    this._next();
  }
  pushData(stream: Readable, chunk: ?(Buffer | string)) : ?boolean {
      if (!super.push(chunk)){
        stream.pause();
      }
  }
  _next() {
    this._currentChunk = this._rarFileChunks.shift();
    if(!this._currentChunk){
      this.push(null);
    } else {
      this._currentChunk.getStream().then((stream) => {
        stream.on('data', (data) => this.pushData(stream, data));
        stream.on('end', () => this._next());
      });
    }
  }
  _read (){
    this.resume();
  }
}
