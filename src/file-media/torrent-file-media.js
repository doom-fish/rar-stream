// @flow
import FileMedia from './file-media';

export default class TorrentFileMedia extends FileMedia {
    constructor(torrentFileInfo: Object) {
        torrentFileInfo.select();
        torrentFileInfo.size = torrentFileInfo.length;
        super(torrentFileInfo);
    }
}
