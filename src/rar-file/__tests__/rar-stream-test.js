//@flow
import test from 'ava';
import RarStream from '../rar-stream';
import RarFileChunk from '../rar-file-chunk';

import MockFileMedia from '../../parsing/__mocks__/mock-file-media';
import streamToBuffer from 'stream-to-buffer';

const streamToBufferPromise = (stream) =>
  new Promise((resolve) =>
    streamToBuffer(stream, (err, buffer) => resolve(buffer)));


test('rar stream should stream over list of file chunks', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const fileMedia = new MockFileMedia(bufferString);
  const RarFileChunk1 = new RarFileChunk(fileMedia,0, 2);
  const RarFileChunk2 = new RarFileChunk(fileMedia,2, 6);
  const rarStream = new RarStream([RarFileChunk1, RarFileChunk2]);
  return streamToBufferPromise(rarStream).then((buffer) => {
    t.deepEqual(buffer, new Buffer(bufferString, 'hex'));
  });
});

test('rar stream should stream over list of file chunks that are fragmented', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const fragmentedResult = '349ABC';
  const fileMedia = new MockFileMedia(bufferString);
  const RarFileChunk1 = new RarFileChunk(fileMedia,1, 2);
  const RarFileChunk2 = new RarFileChunk(fileMedia,4, 6);
  const rarStream = new RarStream([RarFileChunk1, RarFileChunk2]);
  return streamToBufferPromise(rarStream)
    .then((buffer) => t.deepEqual(buffer, new Buffer(fragmentedResult, 'hex')));
});

test('rar stream should stream over longer list of file chunks', (t) => {
  t.plan(1);
  const bufferString ='123456789ABC';
  const fileMedia = new MockFileMedia(bufferString);
  const RarFileChunk1 = new RarFileChunk(fileMedia,0, 2);
  const RarFileChunk2 = new RarFileChunk(fileMedia,2, 4);
  const RarFileChunk3 = new RarFileChunk(fileMedia,4, 6);
  const rarStream = new RarStream([RarFileChunk1, RarFileChunk2, RarFileChunk3]);
  return streamToBufferPromise(rarStream)
  .then((buffer) => t.deepEqual(buffer, new Buffer(bufferString, 'hex')));
});
