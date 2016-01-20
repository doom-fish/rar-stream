
export default class File {
  constructor(options) {
    if (!options) {
      throw Error("Invalid Arguments, file needs options to be passed to its constructor");
    }
    if (typeof options.size === "undefined" || options.size < 0) {
      throw Error("Invalid Arguments, file needs a positive size as options to its constructor");
    }
    if (typeof options.name !== "string") {
      throw Error("Invalid Arguments, file needs a name string as options to its constructor");
    }
    this._name = options.name;
    this._size = options.size;
  }
  get size() {
    return this._size;
  }
  get name() {
    return this._name;
  }
}
