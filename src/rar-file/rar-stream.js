//@flow
import {Readable} from 'stream';
import RarFileChunk from './rar-file-chunk';
export default class RarStream extends Readable {
  _RarFileChunks: RarFileChunk[];
  _byteOffset: number = 0;
  _currentChunk: RarFileChunk;
  constructor(RarFileChunks: RarFileChunk[], options: Object = {}){
    super(options);
    this._RarFileChunks = RarFileChunks;
    this._next();
  }
  pushData(stream: Readable, chunk: ?(Buffer | string)) : ?boolean {
      if (!super.push(chunk)){
        stream.pause();
      }
  }
  _next() {
    this._currentChunk = this._RarFileChunks.shift();
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
