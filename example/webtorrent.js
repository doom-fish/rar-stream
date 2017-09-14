require('events').prototype.setMaxListeners(Infinity);
const prettySeconds = require('pretty-seconds');
const prettyBytes = require('pretty-bytes');
const Webtorrent = require('webtorrent');
const fs = require('fs');
const progressStream = require('progress-stream');

const RarStreamPackage = require('rar-stream');

const client = new Webtorrent();

const magnetURI = process.argv[2];

client.add(magnetURI, torrent => {
  // Got torrent metadata!
  console.log('Client is downloading:', torrent.infoHash);

  const rarStreamPackage = new RarStreamPackage(torrent.files);

  rarStreamPackage.on('file-parsed', file =>
    console.log(`Parsed file: ${file.name}`)
  );

  const innerFiles = rarStreamPackage.getFiles().then(innerFiles => {
    const [innerFile] = innerFiles.filter(
      inner => inner.name.indexOf('mkv') !== -1
    );

    const streamProgress = progressStream({
      length: innerFile.size,
      time: 100,
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
    const fileStream = innerFile
      .createReadStream({
        start: 0,
        end: innerFile.length - 1,
      })
      .pipe(streamProgress)
      .pipe(fs.createWriteStream(innerFile.name));

    fileStream.on('close', () => process.exit());
  });
});
