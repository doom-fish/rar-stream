//@flow
import {Readable} from 'stream';

export class MockFileMedia extends Readable {
  _object: ?Object;
  constructor(object: Object, options: Object) {
    super(options);
    this._object = object;
  }
  _read () {
    this.push(this._object);
    this._object = null;
  }
}

export const mockStreamFromString = (str: string, options: Object = {}, variant: any = 'hex') => {
  if (options.size) {
    let padding = Math.abs(options.size - str.length / 2);
    str += Array(padding).fill().map(() => '00').join('');
  }
  return new MockFileMedia(new Buffer(str, variant), options);
};
