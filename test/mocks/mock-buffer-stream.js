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

export const mockStreamFromString = function (str, options = { padding: 500 }, variant = "hex") {
  if (options.padding) {
    str += Array.apply(0, Array(options.padding)).map(() => "0").join("");
  }
  return new MockFileMedia(new Buffer(str, variant), options);
};
