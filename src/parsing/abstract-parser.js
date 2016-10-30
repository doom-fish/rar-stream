// @flow
import {Readable} from 'stream'

export default class AbstractParser {
  _stream: Readable;
  constructor (stream: Readable) {
    this._stream = stream
  }
  get bytesToRead (): number {
    throw Error('Abstract Getter, implement in sub classes')
  }
  parse () : Object {
    throw Error('Abstract Method, implement in sub classes')
  }
  read (): ?string | ?Buffer {
    return this._stream.read(this.bytesToRead)
  }
}
