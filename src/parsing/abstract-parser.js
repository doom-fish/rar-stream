export default class AbstractParser {
  constructor(buffer) {
    if (!(buffer instanceof Buffer)) {
      throw Error("Invalid Arguments, buffer needs to be a Buffer instance");
    }
    this._buffer = buffer;
  }
  parse() {
    throw Error("Abstract Method, implement in sub classes");
  }
}
