require('events').prototype.setMaxListeners(Infinity);
const prettySeconds = require('pretty-seconds');
const prettyBytes = require('pretty-bytes');
const Webtorrent = require('webtorrent');
const fs = require('fs');
const progressStream = require('progress-stream');

const { RarFileBundle, RarManifest } = require('rar-stream');

const TorrentFileMedia = require(
    'rar-stream/dist/file-media/torrent-file-media'
).default;

const client = new Webtorrent();

const magnetURI = process.argv[2];

client.add(magnetURI, torrent => {
    // Got torrent metadata!
    console.log('Client is downloading:', torrent.infoHash);

    const bundle = new RarFileBundle(
        torrent.files.map(file => new TorrentFileMedia(file))
    );

    const manifest = new RarManifest(bundle);
    manifest.on('file-parsed', file =>
        console.log(`Parsed file: ${file.name}`));
    const innerFiles = manifest.getFiles().then(innerFiles => {
        const innerFile = innerFiles.filter(inner => {
            return inner.name.indexOf('mkv') !== -1;
        })[0];

        const streamProgress = (0, progressStream)({
            length: innerFile.size,
            time: 100
        });
        streamProgress.on('progress', ({ percentage, speed, eta }) => {
            console.log('\x1b[2J\x1b[0f\u001b[0;0H');
            console.log('Downloading', innerFile.name);
            console.log(
                Math.round(percentage) + '%',
                prettyBytes(speed) + '/s',
                prettySeconds(eta) + ' left'
            );
        });
        innerFile
            .createReadStream({ start: 0, end: innerFile.size })
            .pipe(streamProgress)
            .pipe(fs.createWriteStream('outstream.mkv'));
    });
});
