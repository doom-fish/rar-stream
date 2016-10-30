//@flow
import FileMedia from './file-media';
import {Readable} from 'stream';
import type {FileInterval, FileInfo} from './file-media';

type TorrentFileInfo = {
  name: string,
  length: number,
  createReadStream: (interval: FileInterval) => Readable
}
export default class TorrentFileMedia extends FileMedia {
  constructor(torrentFileInfo: TorrentFileInfo) {
    const  {name, length, createReadStream} = torrentFileInfo
    super({name, size: length, createReadStream});
  }
}
