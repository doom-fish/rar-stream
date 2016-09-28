//@flow
import test from 'ava';
import path from 'path';
import fs from 'fs';
import RarManifest from '../rar-manifest';
import RarFileBundle from '../../rar-file/rar-file-bundle'
import LocalFileMedia from '../../file-media/local-file-media';
import streamToBuffer from 'stream-to-buffer';


const streamToBufferPromise = (stream) => new Promise((resolve, reject) => streamToBuffer(stream,
  (err, buffer) => err? reject(err) : resolve(buffer))
);

let fixturePath = path.join(__dirname, '../__fixtures__');
if(global.isBeingRunInWallaby) {
  fixturePath = global.fixturePath;
}

const singleFileBinFixture = fs.readFileSync(
  path.join(fixturePath, 'single1.bin')
);

const multi1FileBinFixture = fs.readFileSync(
  path.join(fixturePath, 'splitted1.bin')
);

const multi2FileBinFixture = fs.readFileSync(
  path.join(fixturePath, 'splitted2.bin')
);

const multi3FileBinFixture = fs.readFileSync(
  path.join(fixturePath, 'splitted3.bin')
);

const createSingleFileRarBundle = () => new RarFileBundle(
  new LocalFileMedia(path.join(fixturePath, 'single.rar'))
);

const createSingleSplittedRarBundle = () => new RarFileBundle(
  new LocalFileMedia(path.join(fixturePath, 'singleSplitted.rar')),
  new LocalFileMedia(path.join(fixturePath, 'singleSplitted.r00')),
  new LocalFileMedia(path.join(fixturePath, 'singleSplitted.r01')),
  new LocalFileMedia(path.join(fixturePath, 'singleSplitted.r02')),
  new LocalFileMedia(path.join(fixturePath, 'singleSplitted.r03'))
);

const createMultipleSingleRarBundle = () => new RarFileBundle(
  new LocalFileMedia(path.join(fixturePath, 'multiple.rar'))
);

const streamsToPromises = (streams) => Promise.all(streams.map(streamToBufferPromise));
const readToEnd = (files) => Promise.all(files.map(file => file.readToEnd()));
const log = (data) => (console.log(data) || data);

const matchRarStreamWithFileSystem = (interval) => (files) => files.map((file) => [
  file.createReadStream(interval),
  fs.createReadStream(path.join(
    fixturePath,
    file.name), interval)
]);
test('single file can be read properly as a whole', (t) => {
  t.plan(2);
  const bundle = createSingleFileRarBundle();
  const manifest = new RarManifest(bundle);
  return manifest.getFiles()
                 .then(readToEnd)
                 .then((buffers) => {
                   t.is(buffers.length, 1);
                   t.deepEqual(singleFileBinFixture, buffers[0]);
                 });
});

test('single file can be read properly as parts', (t) => {
  t.plan(2);
  const bundle = createSingleFileRarBundle();
  const interval = {start: 50, end: 1000};
  const manifest = new RarManifest(bundle);

  return manifest.getFiles()
                .then(matchRarStreamWithFileSystem(interval))
                .then((pairs) => pairs.map(streamsToPromises))
                .then((awaits) => Promise.all(awaits))
                .then(([buffers]) => {
                    t.is(buffers[0].length, buffers[1].length)
                    t.deepEqual(buffers[0], buffers[1]);
                  });
});

test('splitted rar file should be read as a whole', (t) => {
  const bundle = createSingleSplittedRarBundle();
  const manifest = new RarManifest(bundle);
  return manifest.getFiles()
                 .then(readToEnd)
                 .then((buffers) => {
                   t.is(buffers.length, 1);
                   t.deepEqual(singleFileBinFixture, buffers[0]);
                 });
});

test('splitted rar file can be read properly as parts', (t) => {
  t.plan(2);
  const bundle = createSingleSplittedRarBundle();
  const interval = {start: 50, end: 1000};
  const manifest = new RarManifest(bundle);


  return manifest.getFiles()
                  .then(matchRarStreamWithFileSystem(interval))
                  .then((pairs) => pairs.map(streamsToPromises))
                  .then((awaits) => Promise.all(awaits))
                  .then(([buffers]) => {
                    t.is(buffers[0].length, buffers[1].length)
                    t.deepEqual(buffers[0], buffers[1]);
                  });
});

test('single rar file with multiple inner files can be read as a whole', (t) => {
  t.plan(4);
  const bundle = createMultipleSingleRarBundle();
  const manifest = new RarManifest(bundle);
  return manifest.getFiles()
                 .then(readToEnd)
                 .then(buffers => {
                   t.is(buffers.length, 3);
                   t.deepEqual(buffers[0], multi1FileBinFixture)
                   t.deepEqual(buffers[1], multi2FileBinFixture)
                   t.deepEqual(buffers[2], multi3FileBinFixture)
                 });
});

test('single rar file with multiple inner files can be read in pieces', (t) => {
  t.plan(3);
  const bundle = createMultipleSingleRarBundle();
  const manifest = new RarManifest(bundle);
  const interval = {start: 1000, end: 3000};
  return manifest.getFiles()
                 .then(matchRarStreamWithFileSystem(interval))
                 .then((pairs) => pairs.map(streamsToPromises))
                 .then((awaits) => Promise.all(awaits))
                 .then(([pair1, pair2, pair3]) => {
                   t.deepEqual(pair1[0], pair1[1])
                   t.deepEqual(pair2[0], pair2[1])
                   t.deepEqual(pair3[0], pair3[1])
                 });
});
