// @flow
import {Readable} from 'stream'
import FileMedia from '../file-media/file-media'
export default class RarFileChunk {
  _fileMedia: FileMedia;
  _startOffset: number;
  _endOffset: number;
  constructor (fileMedia: FileMedia, startOffset: number, endOffset: number) {
    this._fileMedia = fileMedia
    this._startOffset = startOffset
    this._endOffset = endOffset
  }
  set startOffset (value: number) {
    this._startOffset = value
  }
  set endOffset (value: number) {
    this._endOffset = value
  }
  get length ():number {
    return Math.abs(this._endOffset - this._startOffset)
  }
  getStream (): Promise<Readable> {
    return this._fileMedia.createReadStream({
      start: this._startOffset,
      end: this._endOffset
    })
  }
}
