//@flow
import test from 'ava';
import path from 'path';
import fs from 'fs';
import RarManifest from '../rar-manifest';
import RarFileBundle from '../../rar-file/rar-file-bundle'
import LocalFileMedia from '../../file-media/local-file-media';

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
  const fileBundle = createMultiplesFileRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles()
                .then((files) => Promise.all(files.map(file => file.readToEnd())))
                .then(buffers => {
                  t.is(buffers.length, 5)
                  t.deepEqual(buffers[0], file1)
                  t.deepEqual(buffers[1], file2)
                  t.deepEqual(buffers[3], file3)
                  t.deepEqual(buffers[4], file4)
                });
});
