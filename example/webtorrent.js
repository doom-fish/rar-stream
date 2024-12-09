import fs from "node:fs";
import Webtorrent from "webtorrent";
import { ProgressBar } from "@opentf/cli-pbar";
import { RarFilesPackage } from "rar-stream";
import prettyBytes from "pretty-bytes";
const client = new Webtorrent();
const magnetURI = process.argv[2];

const pBar = new ProgressBar({
  prefix: "Downloading:",
});

client.add(magnetURI, { path: "." }, async (torrent) => {
  // Got torrent metadata!
  console.log("Client is downloading:", torrent.infoHash);

  const rarStreamPackage = new RarFilesPackage(torrent.files);

  rarStreamPackage.on("file-parsed", (file) =>
    console.log(`Parsed file: ${file.name}`),
  );

  const innerFiles = await rarStreamPackage.parse();

  const [innerFile] = innerFiles.filter(
    (inner) => inner.name.indexOf("mkv") !== -1,
  );
  console.log(`Found file: ${innerFile.name}`);

  pBar.start({ total: innerFile.length - 1 });
  const fileStream = innerFile
    .createReadStream({
      start: 0,
      end: innerFile.length - 1,
    })
    .pipe(fs.createWriteStream(innerFile.name));
  setInterval(() => {
    pBar.update({
      value: fileStream.bytesWritten,
      suffix: `${prettyBytes(client.downloadSpeed, {space: false})}/s`,
      
    });
  }, 500);

  fileStream.on("close", () => {
    pBar.stop();
    process.exit();
  });
});
