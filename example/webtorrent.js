var WebTorrent = require('webtorrent');
var fs = require('fs');

var RarFileBundle = require('rar-stream/dist/rar-file/rar-file-bundle').default;
var TorrentFile = require(
    'rar-stream/dist/file-media/torrent-file-media'
).default;
var Manifest = require('rar-stream/dist/rar-manifest/rar-manifest').default;
var client = new WebTorrent();

var magnetURI = '../Arrival.2016.1080p.BluRay.x264-SPARKS.torrent';

client.add(magnetURI, function(torrent) {
    // Got torrent metadata!
    console.log('Client is downloading:', torrent.infoHash);

    const bundle = new RarFileBundle(
        ...torrent.files.map(file => new TorrentFile(file))
    );
    const manifest = new Manifest(bundle);

    manifest.getFiles().then(file => {
        const innerFile = file.filter(
            inner => inner.name.indexOf('mkv') !== -1
        )[0];

        innerFile
            .createReadStream({ start: 0, end: innerFile.size })
            .pipe(fs.createWriteStream('outstream.mkv'));
    });
});
