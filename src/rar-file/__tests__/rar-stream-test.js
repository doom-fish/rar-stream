// @flow
import test from 'ava';
import RarStream from '../rar-stream';
import RarFileChunk from '../rar-file-chunk';

import MockFileMedia from '../../parsing/__mocks__/mock-file-media';
import streamToBuffer from 'stream-to-buffer';

const streamToBufferPromise = (stream) =>
  new Promise((resolve, reject) =>
    streamToBuffer(stream, (err, buffer) => err ? reject(err) : resolve(buffer)));

test('rar stream should stream over list of file chunks', async (t) => {
    const bufferString = '123456789ABC';
    const fileMedia = new MockFileMedia(bufferString);

    const rarStream = new RarStream([
        new RarFileChunk(fileMedia, 0, 2),
        new RarFileChunk(fileMedia, 2, 6)
    ]);
    const buffer = await streamToBufferPromise(rarStream);
    t.deepEqual(buffer, new Buffer(bufferString, 'hex'));
});

test('rar stream should stream over list of file chunks that are fragmented', async (t) => {
    const bufferString = '123456789ABC';
    const fragmentedResult = '349ABC';
    const fileMedia = new MockFileMedia(bufferString);

    const rarStream = new RarStream([
        new RarFileChunk(fileMedia, 1, 2),
        new RarFileChunk(fileMedia, 4, 6)
    ]);
    const buffer = await streamToBufferPromise(rarStream);
    t.deepEqual(buffer, new Buffer(fragmentedResult, 'hex'));
});

test('rar stream should stream over longer list of file chunks', async (t) => {
    const bufferString = '123456789ABC';
    const fileMedia = new MockFileMedia(bufferString);

    const rarStream = new RarStream([
        new RarFileChunk(fileMedia, 0, 2),
        new RarFileChunk(fileMedia, 2, 4),
        new RarFileChunk(fileMedia, 4, 6)
    ]);

    const buffer = await streamToBufferPromise(rarStream);
    t.deepEqual(buffer, new Buffer(bufferString, 'hex'));
});
