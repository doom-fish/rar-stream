import AbstractFileMedia from "./abstract-file-media";

export default class FileMedia extends AbstractFileMedia {
  constructor(fileInfo) {
    if (!fileInfo) {
      throw new Error("Invalid Arguments, fileInfo need to be passed to the constructor");
    }

    super();
    this._createReadStream = fileInfo.createReadStream;
    this._name = fileInfo.name;
    this._size = fileInfo.size;
  }
  get name() {
    return this._name;
  }
  get size() {
    return this._size;
  }
  createReadStream(start, end) {
    if (start > end) {
      throw Error("Invalid Arguments, start offset can not be greater than end offset");
    }
    let stream = this._createReadStream(start, end);

    return new Promise((resolve, reject) => {
      stream.on("readable", () => resolve(stream));
      stream.on("error", (error) => reject(error));
    });
  }
}
