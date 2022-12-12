require("events").prototype.setMaxListeners(Infinity);

const Webtorrent = require("webtorrent");
const fs = require("fs");

const { RarFilesPackage } = require("rar-stream");

const client = new Webtorrent();
const magnetURI = process.argv[2];

client.add(magnetURI, async (torrent) => {
  // Got torrent metadata!
  console.log("Client is downloading:", torrent.infoHash);

  const rarStreamPackage = new RarFilesPackage(torrent.files);

  rarStreamPackage.on("file-parsed", (file) =>
    console.log(`Parsed file: ${file.name}`)
  );

  const innerFiles = await rarStreamPackage.parse();

  const [innerFile] = innerFiles.filter(
    (inner) => inner.name.indexOf("mkv") !== -1
  );

  const fileStream = innerFile
    .createReadStream({
      start: 0,
      end: innerFile.length - 1,
    })

    .pipe(fs.createWriteStream(innerFile.name));

  fileStream.on("close", () => process.exit());
});
