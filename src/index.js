import fs from 'fs';
import path from 'path';
import LocalFileMedia from './file-media/local-file-media';
import RarFileBundle from './rar-file/rar-file-bundle';
import RarManifest from './rar-manifest/rar-manifest';
import progressStream from 'progress-stream';
import prettysize from 'prettysize';

const directory = process.argv[2];

fs.readdir(directory, (err, files) => {
  if (err) console.error(err);
  const fileMedias = files.map((file) => new LocalFileMedia(path.resolve(directory, file)));


  const bundle = new RarFileBundle(...fileMedias);
  const manifest = new RarManifest(bundle);

  manifest.getFiles().then((innerFiles) => {
    innerFiles.forEach(innerFile => {
      const str = progressStream({
        length: innerFile.size,
        time: 100
      });
      str.on('progress', (progress) => {
        process.stdout.write('\x1B[2J\x1B[0f');
        console.log(`Unpacking: ${innerFile.name} (${Math.round(progress.percentage)}%) at speed: ${prettysize(progress.speed)}/S`)
      });

      const writeStream = fs.createWriteStream(path.resolve(directory, innerFile.name));
      innerFile.createReadStream({
        start: 0,
        end: innerFile.size - 1
      }).pipe(str).pipe(writeStream);
    });
  });
});
