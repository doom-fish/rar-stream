// @flow
import FileMedia from './file-media';
type TorrentFileInfo = {
    select(): void,
    createReadStream(): stream$Readable,
    size: number,
    length: number,
    name: string
};
export default class TorrentFileMedia extends FileMedia {
    constructor(torrentFileInfo: TorrentFileInfo) {
        torrentFileInfo.select();
        torrentFileInfo.size = torrentFileInfo.length;
        super(torrentFileInfo);
    }
}
