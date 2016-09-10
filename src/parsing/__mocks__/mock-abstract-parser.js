import AbstractParser from '../abstract-parser';

export default class MockAbstractParser extends AbstractParser {
  constructor(stream, size) {
    super(stream);
    this._size = size;
  }
  get bytesToRead() {
    return this._size;
  }
}
