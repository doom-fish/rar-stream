import fs from 'fs';
import path from 'path';
import LocalFileMedia from './file-media/local-file-media';
import RarFileBundle from './rar-file/rar-file-bundle';
import RarManifest from './rar-manifest/rar-manifest';

const directory = process.argv[2];

fs.readdir(directory, (err, files) => {
  if(err) console.error(err);
  const fileMedias = files.map((file) => new LocalFileMedia(path.resolve(directory, file)));
  // console.log(fileMedias);

  const bundle = new RarFileBundle(...fileMedias);
  const manifest = new RarManifest(bundle);

  manifest.getFiles().then((innerFiles) => {
    innerFiles.forEach(innerFile => {
      const writeStream = fs.createWriteStream(path.resolve(directory, innerFile.name));
      innerFile.createReadStream(0, innerFile.size - 1).pipe(writeStream);
    });
  });
});
