//@flow
import test from 'ava';
import RarFile from '../rar-file';
import FileChunk from '../file-chunk';
import MockFileMedia from '../../parsing/__mocks__/mock-file-media';
import streamToBuffer from 'stream-to-buffer';

const streamToBufferPromise = (stream) =>
  new Promise((resolve) =>
    streamToBuffer(stream, (err, buffer) => resolve(buffer)));

test('RarFile#createReadStream should return a rar-stream that is composed by chunks', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const fileMedia = new MockFileMedia(bufferString);
  const fileChunk1 = new FileChunk(fileMedia,0, 3);
  const fileChunk2 = new FileChunk(fileMedia,3, 6);
  const rarFile = new RarFile([fileChunk1, fileChunk2]);
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
  const fileChunk1 = new FileChunk(fileMedia,0, 3);
  const fileChunk2 = new FileChunk(fileMedia,3, 6);
  const rarFile = new RarFile([fileChunk1, fileChunk2]);
  const stream = rarFile.createReadStream(1, 5);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});

test('RarFile#createReadStream should drop chunks depending on end offsets', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='123456';

  const fileMedia = new MockFileMedia(bufferString);
  const fileChunk1 = new FileChunk(fileMedia,0, 1);
  const fileChunk2 = new FileChunk(fileMedia,1, 2);
  const fileChunk3 = new FileChunk(fileMedia,2, 3);
  const fileChunk4 = new FileChunk(fileMedia, 3, 6);

  const rarFile = new RarFile([
    fileChunk1,
    fileChunk2,
    fileChunk3,
    fileChunk4
  ]);
  const stream = rarFile.createReadStream(0, 3);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});


test('RarFile#createReadStream should drop chunk depending on start offset', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='789ABC';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile([
    new FileChunk(fileMedia,0, 1),
    new FileChunk(fileMedia,1, 2),
    new FileChunk(fileMedia,2, 3),
    new FileChunk(fileMedia, 3, 6)
  ]);
  const stream = rarFile.createReadStream(3, 6);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});

test('RarFile#createReadStream should drop chunk depending on start offset', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='3456789A';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile([new FileChunk(fileMedia, 0, 6)]);
  const stream = rarFile.createReadStream(1, 5);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});

test('RarFile#createReadStream should drop chunk depending on start offset', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const shortnedResult ='3456789A';
  const fileMedia = new MockFileMedia(bufferString);
  const rarFile = new RarFile([
    new FileChunk(fileMedia, 0, 1),
    new FileChunk(fileMedia, 1, 2),
    new FileChunk(fileMedia, 2, 3),
    new FileChunk(fileMedia, 3, 4),
    new FileChunk(fileMedia, 4, 5),
    new FileChunk(fileMedia, 5, 6),
  ]);
  const stream = rarFile.createReadStream(1, 5);
  return streamToBufferPromise(stream)
    .then((buffer) => t.deepEqual(new Buffer(shortnedResult, 'hex'), buffer));
});
