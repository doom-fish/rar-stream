//
const test = require('ava');
const path = require('path');
const fs = require('fs');
const RarManifest = require('../rar-manifest');

const LocalFileMedia = require('../../file-media/local-file-media');
const { streamToBuffer } = require('../../stream-utils');
const makeRarFileBundle = require('../../rar-file/rar-file-bundle');
let fixturePath = path.join(__dirname, '../__fixtures__');

if (global.isBeingRunInWallaby) {
  fixturePath = global.fixturePath;
}

const singleFilePath = path.join(fixturePath, 'single/single.txt');
const multiFilePath = path.join(fixturePath, 'multi/multi.txt');

const singleSplitted1FilePath = path.join(
  fixturePath,
  'single-splitted/splitted1.txt'
);
const singleSplitted2FilePath = path.join(
  fixturePath,
  'single-splitted/splitted2.txt'
);
const singleSplitted3FilePath = path.join(
  fixturePath,
  'single-splitted/splitted3.txt'
);

const multiSplitted1FilePath = path.join(
  fixturePath,
  'multi-splitted/splitted1.txt'
);
const multiSplitted2FilePath = path.join(
  fixturePath,
  'multi-splitted/splitted2.txt'
);
const multiSplitted3FilePath = path.join(
  fixturePath,
  'multi-splitted/splitted3.txt'
);
const multiSplitted4FilePath = path.join(
  fixturePath,
  'multi-splitted/splitted4.txt'
);

const createSingleFileRar = () => [
  new LocalFileMedia(path.join(fixturePath, 'single/single.rar')),
];

const createSingleRarWithManyInner = () => [
  new LocalFileMedia(
    path.join(fixturePath, 'single-splitted/single-splitted.rar')
  ),
];

const createMultipleRarFileWithOneInner = () => [
  new LocalFileMedia(path.join(fixturePath, 'multi/multi.rar')),
  new LocalFileMedia(path.join(fixturePath, 'multi/multi.r01')),
  new LocalFileMedia(path.join(fixturePath, 'multi/multi.r00')),
];

const createMultipleRarFileWithManyInner = () => [
  new LocalFileMedia(
    path.join(fixturePath, 'multi-splitted/multi-splitted.rar')
  ),
  new LocalFileMedia(
    path.join(fixturePath, 'multi-splitted/multi-splitted.r00')
  ),
  new LocalFileMedia(
    path.join(fixturePath, 'multi-splitted/multi-splitted.r01')
  ),
];

const readToEnd = f => Promise.all(f.map(file => file.readToEnd()));

test('rar manifest emits events for when parsing ends', async t => {
  const bundle = createMultipleRarFileWithOneInner();
  t.plan(1);
  const manifest = new RarManifest(bundle);
  let eventResult;
  manifest.on('parsing-end', files => {
    eventResult = files;
  });
  const files = await manifest.getFiles();
  t.is(eventResult, files);
});

test('rar manifest emits events for when parsing starts', async t => {
  const files = createMultipleRarFileWithOneInner();
  t.plan(1);
  const manifest = new RarManifest(files);

  manifest.on('parsing-start', manifest => {
    t.deepEqual(manifest, manifest);
  });
  await manifest.getFiles();
});

test('rar manifest emits events for each parsed file', async t => {
  const files = createMultipleRarFileWithOneInner();
  const bundle = makeRarFileBundle(files);
  t.plan(files.length);
  const manifest = new RarManifest(files);
  let i = 0;
  manifest.on('file-parsed', file => {
    t.is(file, bundle.files[i++]);
  });
  await manifest.getFiles();
});

test('single rar file with one inner file can be read as whole', async t => {
  const bundle = createSingleFileRar();
  const manifest = new RarManifest(bundle);
  const files = await manifest.getFiles();
  const [rarFileContent] = await readToEnd(files);
  const singleFileContent = fs.readFileSync(singleFilePath);

  t.is(rarFileContent.length, singleFileContent.length);
  t.deepEqual(rarFileContent, singleFileContent);
});

test('single rar file with one inner files can be read in parts', async t => {
  const interval = { start: 53, end: 1000 };

  const bundle = createSingleFileRar();
  const manifest = new RarManifest(bundle);

  const [file] = await manifest.getFiles();
  const rarFileInterval = file.createReadStream(interval);
  const singleFileInterval = fs.createReadStream(singleFilePath, interval);
  const rarFileBuffer = await streamToBuffer(rarFileInterval);
  const singleFileBuffer = await streamToBuffer(singleFileInterval);

  t.is(rarFileBuffer.length, singleFileBuffer.length);
  t.deepEqual(rarFileBuffer, singleFileBuffer);
});

test('single rar file with many inner files can be read as whole', async t => {
  const bundle = createSingleRarWithManyInner();
  const manifest = new RarManifest(bundle);
  const [rarFile1, rarFile2, rarFile3] = await manifest
    .getFiles()
    .then(readToEnd);

  const splitted1 = fs.readFileSync(singleSplitted1FilePath);
  const splitted2 = fs.readFileSync(singleSplitted2FilePath);
  const splitted3 = fs.readFileSync(singleSplitted3FilePath);

  t.is(rarFile1.length, splitted1.length);
  t.is(rarFile2.length, splitted2.length);
  t.is(rarFile3.length, splitted3.length);

  t.deepEqual(rarFile1, splitted1);
  t.deepEqual(rarFile2, splitted2);
  t.deepEqual(rarFile3, splitted3);
});

test('single rar file with many inner files can be read in parts', async t => {
  const bundle = createSingleRarWithManyInner();
  const interval = { start: 50, end: 200 };
  const manifest = new RarManifest(bundle);

  const [rarFile1, rarFile2, rarFile3] = await manifest.getFiles();

  const rarFile1Buffer = await streamToBuffer(
    rarFile1.createReadStream(interval)
  );
  const rarFile2Buffer = await streamToBuffer(
    rarFile2.createReadStream(interval)
  );
  const rarFile3Buffer = await streamToBuffer(
    rarFile3.createReadStream(interval)
  );

  const splittedFile1Buffer = await streamToBuffer(
    fs.createReadStream(singleSplitted1FilePath, interval)
  );
  const splittedFile2Buffer = await streamToBuffer(
    fs.createReadStream(singleSplitted2FilePath, interval)
  );
  const splittedFile3Buffer = await streamToBuffer(
    fs.createReadStream(singleSplitted3FilePath, interval)
  );

  t.is(rarFile1Buffer.length, splittedFile1Buffer.length);
  t.is(rarFile2Buffer.length, splittedFile2Buffer.length);
  t.is(rarFile3Buffer.length, splittedFile3Buffer.length);

  t.deepEqual(rarFile1Buffer, splittedFile1Buffer);
  t.deepEqual(rarFile2Buffer, splittedFile2Buffer);
  t.deepEqual(rarFile3Buffer, splittedFile3Buffer);
});
//
test('multiple rar file with one inner can be read as a whole', async t => {
  const bundle = createMultipleRarFileWithOneInner();
  const manifest = new RarManifest(bundle);
  const [rarFileBuffer] = await manifest.getFiles().then(readToEnd);
  const multiFile = fs.readFileSync(multiFilePath);
  t.is(rarFileBuffer.length, multiFile.length);
  t.deepEqual(rarFileBuffer, multiFile);
});

test('multiple rar file with one inner can be read as in parts', async t => {
  const interval = { start: 0, end: 100 };

  const bundle = createMultipleRarFileWithOneInner();
  const manifest = new RarManifest(bundle);

  const [file] = await manifest.getFiles();
  const rarFileBuffer = await streamToBuffer(file.createReadStream(interval));
  const multiFileBuffer = await streamToBuffer(
    fs.createReadStream(multiFilePath, interval)
  );

  t.is(rarFileBuffer.length, multiFileBuffer.length);
  t.deepEqual(rarFileBuffer, multiFileBuffer);
});

test('multi rar file with many inner files can be read as whole', async t => {
  const bundle = createMultipleRarFileWithManyInner();
  const manifest = new RarManifest(bundle);
  const [
    rarFile1,
    rarFile2,
    rarFile3,
    rarFile4,
  ] = await manifest.getFiles().then(readToEnd);

  const splitted1 = fs.readFileSync(multiSplitted1FilePath);
  const splitted2 = fs.readFileSync(multiSplitted2FilePath);
  const splitted3 = fs.readFileSync(multiSplitted3FilePath);
  const splitted4 = fs.readFileSync(multiSplitted4FilePath);

  t.is(rarFile1.length, splitted1.length);
  t.is(rarFile2.length, splitted2.length);
  t.is(rarFile3.length, splitted3.length);
  t.is(rarFile4.length, splitted4.length);
});

test('multi rar file with many inner files can be read in parts', async t => {
  const bundle = createMultipleRarFileWithManyInner();
  const interval = { start: 56, end: 200 };
  const manifest = new RarManifest(bundle);

  const [rarFile1, rarFile2, rarFile3, rarFile4] = await manifest.getFiles();

  const rarFile1Buffer = await streamToBuffer(
    rarFile1.createReadStream(interval)
  );
  const rarFile2Buffer = await streamToBuffer(
    rarFile2.createReadStream(interval)
  );
  const rarFile3Buffer = await streamToBuffer(
    rarFile3.createReadStream(interval)
  );
  const rarFile4Buffer = await streamToBuffer(
    rarFile4.createReadStream(interval)
  );

  const splittedFile1Buffer = await streamToBuffer(
    fs.createReadStream(multiSplitted1FilePath, interval)
  );
  const splittedFile2Buffer = await streamToBuffer(
    fs.createReadStream(multiSplitted2FilePath, interval)
  );
  const splittedFile3Buffer = await streamToBuffer(
    fs.createReadStream(multiSplitted3FilePath, interval)
  );
  const splittedFile4Buffer = await streamToBuffer(
    fs.createReadStream(multiSplitted4FilePath, interval)
  );

  t.is(rarFile1Buffer.length, splittedFile1Buffer.length);
  t.is(rarFile2Buffer.length, splittedFile2Buffer.length);
  t.is(rarFile3Buffer.length, splittedFile3Buffer.length);
  t.is(rarFile4Buffer.length, splittedFile4Buffer.length);

  t.deepEqual(rarFile1Buffer, splittedFile1Buffer);
  t.deepEqual(rarFile2Buffer, splittedFile2Buffer);
  t.deepEqual(rarFile3Buffer, splittedFile3Buffer);
  t.deepEqual(rarFile4Buffer, splittedFile4Buffer);
});
