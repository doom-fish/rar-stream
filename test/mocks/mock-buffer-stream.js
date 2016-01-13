import {Readable} from "stream";

export class MockFileMedia extends Readable {
  constructor(object, options) {
    super(options);
    this._object = object;
  }
}

MockFileMedia.prototype._read = function () {
  this.push(this._object);
  this._object = null;
};

export const mockStreamFromString = function (str, options = {}, variant = "hex") {
  if (options.size) {
    let padding = Math.abs(options.size - str.length / 2);
    str += Array.apply(0, Array(padding)).map(() => "00").join("");
  }
  return new MockFileMedia(new Buffer(str, variant), options);
};
