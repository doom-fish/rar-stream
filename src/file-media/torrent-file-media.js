module.exports = class TorrentFileMedia {
    constructor(torrentFileInfo) {
        this.size = torrentFileInfo.length;
        this.name = torrentFileInfo.name;
    }
    createReadStream(interval) {
        const stream = createReadStream(interval);
        return new Promise((resolve, reject) => {
            stream.once('readable', () => resolve(stream));
            stream.on('error', reject);
        });
    }
};
