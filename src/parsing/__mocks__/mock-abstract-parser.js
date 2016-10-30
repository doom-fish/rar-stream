// @flow
import {Readable} from 'stream'
import AbstractParser from '../abstract-parser'

export default class MockAbstractParser extends AbstractParser {
  _size: number;
  constructor (stream: Readable, size: number) {
    super(stream)
    this._size = size
  }
  get bytesToRead () :number {
    return this._size
  }
}
