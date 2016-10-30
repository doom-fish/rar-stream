// @flow
import FileMedia from './file-media'
import {Readable} from 'stream'
import type {FileInterval} from './file-media'  // eslint-disable-line 

type TorrentFileInfo = {
  name: string,
  length: number,
  createReadStream: (interval: FileInterval) => Readable
}

export default class TorrentFileMedia extends FileMedia {
  constructor ({name, length, createReadStream}: TorrentFileInfo) {
    super({name, size: length, createReadStream})
  }
}
