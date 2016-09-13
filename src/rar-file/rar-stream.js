//@flow
import {Readable} from 'stream';
import FileChunk from './file-chunk';
export default class RarStream extends Readable {
  _fileChunks: FileChunk[];
  _byteOffset: number = 0;
  _currentChunk: FileChunk;
  constructor(fileChunks: FileChunk[], options: Object = {}){
    super(options);
    this._fileChunks = fileChunks;
    this._next();
  }
  pushData(stream: Readable, chunk: ?(Buffer | string)) : ?boolean {
      if (!super.push(chunk)){
        stream.pause();
      }
  }
  _next() {
    this._currentChunk = this._fileChunks.shift();
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
