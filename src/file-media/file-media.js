//@flow
import {Readable} from 'stream';
export default class FileMedia {
  // eslint-disable-next-line
  _createReadStream: (start: number, end: number) => Readable;
  _name: string;
  _size: number;
  constructor(fileInfo: Object) {
    if (!fileInfo) {
      throw new Error('Invalid Arguments, fileInfo need to be passed to the constructor');
    }

    this._createReadStream = fileInfo.createReadStream;
    this._name = fileInfo.name;
    this._size = fileInfo.size;
  }
  get name() : string {
    return this._name;
  }
  get size() : number {
    return this._size;
  }
  createReadStream(start: number, end: number) {
    if (start > end) {
      throw Error('Invalid Arguments, start offset can not be greater than end offset');
    }
    let stream = this._createReadStream(start, end);

    return new Promise((resolve, reject) => {
      stream.on('readable', () => resolve(stream));
      stream.on('error', (error) => reject(error));
    });
  }
}
