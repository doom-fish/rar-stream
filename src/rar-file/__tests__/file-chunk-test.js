//@flow
import streamToBuffer from 'stream-to-buffer';
import test from 'ava';
import MockFileMedia from '../../parsing/__mocks__/mock-file-media';
import FileChunk from '../file-chunk'

const streamToBufferPromise = (stream) =>
  new Promise((resolve) =>
    streamToBuffer(stream, (err, buffer) =>
      resolve(buffer)));

test('FileChunk#getStream should return a stream from its FileMedia', t => {
  t.plan(1);
  const bufferString ='123456789A';
  const fileMedia = new MockFileMedia(bufferString);
  const fileChunk = new FileChunk(fileMedia, 0, 5);
  return fileChunk.getStream()
                  .then(streamToBufferPromise)
                  .then((buffer) => {
                    t.deepEqual(new Buffer(bufferString, 'hex'), buffer);
                  });
});

test('FileChunk#getStream should return a stream with a subset stream of FileMedia', t => {
  t.plan(1);
  const bufferString ='123456789A';
  const fileMedia = new MockFileMedia(bufferString);
  const fileChunk = new FileChunk(fileMedia, 2, 5);
  return fileChunk.getStream()
                  .then(streamToBufferPromise)
                  .then((buffer) => {
                    t.deepEqual(new Buffer('56789A', 'hex'), buffer);
                  });
});

test('FileChunk#getStream should return a stream with another subset stream of FileMedia', t => {
  t.plan(1);
  const bufferString ='123456789A';
  const fileMedia = new MockFileMedia(bufferString);
  const fileChunk = new FileChunk(fileMedia, 1, 3);
  return fileChunk.getStream()
                  .then(streamToBufferPromise)
                  .then((buffer) => {
                    t.deepEqual(new Buffer('3456', 'hex'), buffer);
                  });
});

test('FileChunk#length should return end - start offset', (t) => {
  const bufferString ='123456789A';
  const fileMedia = new MockFileMedia(bufferString);
  let fileChunk = new FileChunk(fileMedia, 1, 3);
  t.is(fileChunk.length, 2);
  fileChunk = new FileChunk(fileMedia, 0, 3);
  t.is(fileChunk.length, 3);
  fileChunk = new FileChunk(fileMedia, 1, 2);
  t.is(fileChunk.length, 1);
  fileChunk = new FileChunk(fileMedia, 0, 5);
  t.is(fileChunk.length, 5);
});
