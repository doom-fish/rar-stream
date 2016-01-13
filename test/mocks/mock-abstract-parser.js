import AbstractParser from "../../src/parsing/abstract-parser";

export default class MockAbstractParser extends AbstractParser {
  constructor(stream, size) {
    super(stream);
    this._size = size;
  }
  get size() {
    return this._size;
  }
}
