//@flow
import test from 'ava';
import RarFile from '../rar-file';
import RarFileChunk from '../rar-file-chunk';
import MockFileMedia from '../../parsing/__mocks__/mock-file-media';
import streamToBuffer from 'stream-to-buffer';

const streamToBufferPromise = (stream) =>
  new Promise((resolve) =>
    streamToBuffer(stream, (err, buffer) => resolve(buffer)));

test('RarFile#createReadStream should return a rar-stream that is composed by chunks', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile('file.txt',
    new RarFileChunk(fileMedia,0, 3),
    new RarFileChunk(fileMedia,3, 6)
  );
  const stream = rarFile.createReadStream(0, 6);
  return streamToBufferPromise(stream)
    .then((buffer) => {
      t.deepEqual(new Buffer(bufferString, 'hex'), buffer);
    });
});

test('RarFile#createReadStream should return a shortened rar-stream that is composed by chunks', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='3456789A';

  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile(
    'file.txt',
    new RarFileChunk(fileMedia,0, 3),
    new RarFileChunk(fileMedia,3, 6)
  );
  const stream = rarFile.createReadStream(1, 5);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});

test('RarFile#createReadStream should drop chunks depending on end offsets', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='123456';

  const fileMedia = new MockFileMedia(bufferString);

  const rarFile = new RarFile(
    'file.txt',
    new RarFileChunk(fileMedia,0, 1),
    new RarFileChunk(fileMedia,1, 2),
    new RarFileChunk(fileMedia,2, 3),
    new RarFileChunk(fileMedia, 3, 6)
  );
  const stream = rarFile.createReadStream(0, 3);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});


test('RarFile#createReadStream should drop chunk depending on start offset', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='789ABC';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile(
    'file.txt',
    new RarFileChunk(fileMedia,0, 1),
    new RarFileChunk(fileMedia,1, 2),
    new RarFileChunk(fileMedia,2, 3),
    new RarFileChunk(fileMedia, 3, 6)
  );
  const stream = rarFile.createReadStream(3, 6);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});
test('RarFile#createReadStream should expose name property', (t) => {
  const rarFile = new RarFile('file.txt');
  t.is(rarFile.name, 'file.txt');
});

test('RarFile#createReadStream should expose size property', (t) => {
  const bufferString ='123456789ABC';
  const fileMedia = new MockFileMedia(bufferString);
  let rarFile = new RarFile('file.txt', new RarFileChunk(fileMedia, 0, 5));
  t.is(rarFile.size, 5);
  rarFile = new RarFile('file.txt', new RarFileChunk(fileMedia, 0, 4));
  t.is(rarFile.size, 4);
  rarFile = new RarFile('file.txt', new RarFileChunk(fileMedia, 2, 4));
  t.is(rarFile.size, 2);
  rarFile = new RarFile('file.txt', new RarFileChunk(fileMedia, 2, 3));
  t.is(rarFile.size, 1);
  rarFile = new RarFile('file.txt',
    new RarFileChunk(fileMedia, 1, 2),
    new RarFileChunk(fileMedia, 2, 3),
    new RarFileChunk(fileMedia, 3, 4),
  );
  t.is(rarFile.size, 3);
});
test('RarFile#createReadStream should drop chunk depending on start offset', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='3456789A';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile('file.txt', new RarFileChunk(fileMedia, 0, 6));
  const stream = rarFile.createReadStream(1, 5);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});

test('RarFile#createReadStream should drop chunk depending on start offset', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='3456789A';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile(
    'file.txt',
    new RarFileChunk(fileMedia, 0, 1),
    new RarFileChunk(fileMedia, 1, 2),
    new RarFileChunk(fileMedia, 2, 3),
    new RarFileChunk(fileMedia, 3, 4),
    new RarFileChunk(fileMedia, 4, 5),
    new RarFileChunk(fileMedia, 5, 6),
  );
  const stream = rarFile.createReadStream(1, 5);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});


test('RarFile#createReadStream should parse a fragmented stream properly', (t) => {
  t.plan(1);
  const bufferString ='123456789ABCDF';
  const shortnedResult ='789a';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile(
    'file.txt',
    new RarFileChunk(fileMedia, 0, 1),
    new RarFileChunk(fileMedia, 3, 5),
    new RarFileChunk(fileMedia, 7, 8)
  );
  const stream = rarFile.createReadStream(1, 3);
  return streamToBufferPromise(stream)
    .then((buffer) => {
      t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer)
    });
});
