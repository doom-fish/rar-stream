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

const createSingleFileRarBundle = () => new RarFileBundle(
  new LocalFileMedia(path.join(fixturePath, 'file.rar'))
);

const file1 = fs.readFileSync(path.join(fixturePath, 'file1.txt'));
const file2 = fs.readFileSync(path.join(fixturePath, 'file2.txt'));
const file3 = fs.readFileSync(path.join(fixturePath, 'file3.txt'));
const file4 = fs.readFileSync(path.join(fixturePath, 'file4.txt'));

const  createMultiplesPartFileRarBundle = () => new RarFileBundle(
  new LocalFileMedia(path.join(fixturePath, 'files1k.part1.rar')),
  new LocalFileMedia(path.join(fixturePath, 'files1k.part2.rar')),
  new LocalFileMedia(path.join(fixturePath, 'files1k.part3.rar')),
  new LocalFileMedia(path.join(fixturePath, 'files1k.part4.rar')),
  new LocalFileMedia(path.join(fixturePath, 'files1k.part5.rar'))
);


const  createMultiples2kFileRarBundle = () => new RarFileBundle(
  new LocalFileMedia(path.join(fixturePath, 'files2k.rar')),
  new LocalFileMedia(path.join(fixturePath, 'files2k.r00')),
  new LocalFileMedia(path.join(fixturePath, 'files2k.r01'))
);

const  createMultiplesFileRarBundle = () => new RarFileBundle(
  new LocalFileMedia(path.join(fixturePath, 'files.rar')),
  new LocalFileMedia(path.join(fixturePath, 'files.r00')),
  new LocalFileMedia(path.join(fixturePath, 'files.r01')),
  new LocalFileMedia(path.join(fixturePath, 'files.r02')),
  new LocalFileMedia(path.join(fixturePath, 'files.r03')),
);

test('file-manifest#getFiles should should a promise with files matching file names of inner rar files', t => {
  t.plan(3);
  const fileBundle = createSingleFileRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles().then((files) => {
    t.is(files.length, 2);
    t.is(files[0].name, 'file1.txt');
    t.is(files[1].name, 'file2.txt');
  })
});

test('file-manifest#getFiles should should a promise with files matching file content of inner rar files', t => {
  t.plan(3);
  const fileBundle = createSingleFileRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles()
                .then((files) => Promise.all(files.map(file => file.readToEnd())))
                .then(buffers => {
                  t.is(buffers.length, 2);
                  t.deepEqual(buffers[0], file1);
                  t.deepEqual(buffers[1], file2);

                });
});

test('file-manifest#getFiles should should a promise with files matching file content of inner rar files', t => {
  t.plan(5);
  const fileBundle = createMultiples2kFileRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles()
                .then((files) => Promise.all(files.map(file => file.readToEnd())))
                .then(buffers => {
                  t.is(buffers.length, 4);
                  t.deepEqual(buffers[0], file1);
                  t.deepEqual(buffers[1], file2);
                  t.deepEqual(buffers[2], file3);
                  t.deepEqual(buffers[3], file4);
                });
});

test('file-manifest#getFiles should should a promise with files matching file content of inner rar files', t => {
  t.plan(5);
  const fileBundle = createMultiplesPartFileRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles()
                .then((files) => Promise.all(files.map(file => file.readToEnd())))
                .then(buffers => {
                  t.is(buffers.length, 4);
                  t.deepEqual(buffers[0], file1);
                  t.deepEqual(buffers[1], file2);
                  t.deepEqual(buffers[2], file3);
                  t.deepEqual(buffers[3], file4);
                });
});

test('file-manifest#getFiles should should a promise with files matching file content of inner rar files', t => {
  t.plan(1);
  const fileBundle = createMultiplesFileRarBundle();
  const manifest = new RarManifest(fileBundle);
  const innerFileBuffer = manifest.getFiles()
                .then((files) => files[1].createReadStream(260, 500))
                .then(streamToBufferPromise);

  const outterFileBuffer = streamToBufferPromise(fs.createReadStream(path.join(fixturePath, 'file2.txt'), {start: 260, end: 500}));

  return Promise.all([innerFileBuffer, outterFileBuffer])
        .then(([buffer1, buffer2]) => {
          t.deepEqual(buffer1.length, buffer2.length);
       });
});

test('file-manifest#getFiles should should a promise with files matching file content of inner rar files', t => {
  t.plan(5);
  const fileBundle = createMultiplesFileRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles()
                .then((files) => Promise.all(files.map(file => file.readToEnd())))
                .then(buffers => {
                  t.is(buffers.length, 4);
                  t.deepEqual(buffers[0], file1);
                  t.deepEqual(buffers[1], file2);
                  t.deepEqual(buffers[2], file3);
                  t.deepEqual(buffers[3], file4);
                });
});
