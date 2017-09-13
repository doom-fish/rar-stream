module.exports = class TorrentFileMedia {
    constructor(torrentFile) {
        this.size = torrentFile.length;
        this.name = torrentFile.name;
        this.torrentFile = torrentFile;
    }
    createReadStream(interval) {
        return this.torrentFile.createReadStream(interval);
    }
};
