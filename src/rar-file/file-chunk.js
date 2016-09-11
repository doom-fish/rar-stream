//@flow
import {Readable} from 'stream';
import FileMedia from '../file-media/file-media';
export default class FileChunk {
  _fileMedia: FileMedia;
  _startOffset: number;
  _endOffset: number;
  constructor(fileMedia: FileMedia, startOffset: number, endOffset: number) {
    this._fileMedia   = fileMedia;
    this._startOffset = startOffset;
    this._endOffset   = endOffset;
  }
  getStream (): Promise<Readable> {
    return this._fileMedia.createReadStream(this._startOffset, this._endOffset);
  }
}
