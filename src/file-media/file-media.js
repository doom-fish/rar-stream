//@flow
import {Readable} from 'stream';
export default class FileMedia {
   _createReadStream: (opts: Object) => Readable;
  _name: string;
  _size: number;
  constructor(fileInfo: Object) {
    this._createReadStream = (opts) => fileInfo.createReadStream(opts);
    this._name = fileInfo.name;
    this._size = fileInfo.size;
  }
  get name() : string {
    return this._name;
  }
  get size() : number {
    return this._size;
  }
  createReadStream({start, end}) : Promise<Readable> {
    if (start > end) {
      throw Error('Invalid Arguments, start offset can not be greater than end offset');
    }
    let stream = this._createReadStream({start, end: end});

    return new Promise((resolve, reject) => {
      stream.on('readable', () => resolve(stream));
      stream.on('error', (error) => reject(error));
    });
  }
}
