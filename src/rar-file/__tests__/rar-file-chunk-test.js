// @flow
import streamToBuffer from 'stream-to-buffer';
import test from 'ava';
import MockFileMedia from '../../parsing/__mocks__/mock-file-media';
import RarFileChunk from '../rar-file-chunk';

const streamToBufferPromise = stream =>
    new Promise((resolve, reject) =>
        streamToBuffer(
            stream,
            (err, buffer) => err ? reject(err) : resolve(buffer)
        ));

test(
    'RarFileChunk#getStream should return a stream from its FileMedia',
    async t => {
        const bufferString = '123456789A';
        const fileMedia = new MockFileMedia(bufferString);
        const rarFileChunk = new RarFileChunk(fileMedia, 0, 5);
        const stream = await rarFileChunk.getStream();
        const buffer = await streamToBufferPromise(stream);
        t.deepEqual(new Buffer(bufferString, 'hex'), buffer);
    }
);

test(
    'RarFileChunk#getStream should return a stream with a subset stream of FileMedia',
    async t => {
        const bufferString = '123456789A';
        const fileMedia = new MockFileMedia(bufferString);
        const rarFileChunk = new RarFileChunk(fileMedia, 2, 5);
        const stream = await rarFileChunk.getStream();
        const buffer = await streamToBufferPromise(stream);
        t.deepEqual(new Buffer('56789A', 'hex'), buffer);
    }
);

test(
    'RarFileChunk#getStream should return a stream with another subset stream of FileMedia',
    async t => {
        const bufferString = '123456789A';
        const fileMedia = new MockFileMedia(bufferString);
        const rarFileChunk = new RarFileChunk(fileMedia, 1, 3);
        const stream = await rarFileChunk.getStream();
        const buffer = await streamToBufferPromise(stream);
        t.deepEqual(new Buffer('3456', 'hex'), buffer);
    }
);

test('RarFileChunk#length should return end - start offset', t => {
    const bufferString = '123456789A';
    const fileMedia = new MockFileMedia(bufferString);
    let rarFileChunk = new RarFileChunk(fileMedia, 1, 3);
    t.is(rarFileChunk.length, 2);
    rarFileChunk = new RarFileChunk(fileMedia, 0, 3);
    t.is(rarFileChunk.length, 3);
    rarFileChunk = new RarFileChunk(fileMedia, 1, 2);
    t.is(rarFileChunk.length, 1);
    rarFileChunk = new RarFileChunk(fileMedia, 0, 5);
    t.is(rarFileChunk.length, 5);
});
