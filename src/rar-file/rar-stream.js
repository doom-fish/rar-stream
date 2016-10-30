// @flow
import {Readable} from 'stream'
import RarFileChunk from './rar-file-chunk'
export default class RarStream extends Readable {
  constructor (...rarFileChunks: RarFileChunk[]) {
    super()
    this._next(rarFileChunks)
  }
  pushData (stream: Readable, chunk: ?(Buffer | string)) : ?boolean {
    if (!super.push(chunk)) {
      stream.pause()
    }
  }
  _next (rarFileChunks: RarFileChunk[]) {
    const chunk = rarFileChunks.shift()
    if (!chunk) {
      this.push(null)
    } else {
      chunk.getStream().then((stream) => {
        stream.on('data', (data) => this.pushData(stream, data))
        stream.on('end', () => this._next(rarFileChunks))
      })
    }
  }
  _read () {
    this.resume()
  }
}
