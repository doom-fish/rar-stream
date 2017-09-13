//
const test = require('ava');
const RarStream = require('../rar-stream');
const RarFileChunk = require('../rar-file-chunk');

const MockFileMedia = require('../../parsing/__mocks__/mock-file-media');
const { streamToBuffer } = require('../../stream-utils');

test('rar stream should stream over list of file chunks', async t => {
    const bufferString = '123456789ABC';
    const fileMedia = new MockFileMedia(bufferString);

    const rarStream = new RarStream([
        new RarFileChunk(fileMedia, 0, 2),
        new RarFileChunk(fileMedia, 2, 6),
    ]);
    const buffer = await streamToBuffer(rarStream);
    t.deepEqual(buffer, new Buffer(bufferString, 'hex'));
});

test('rar stream should stream over list of file chunks that are fragmented', async t => {
    const bufferString = '123456789ABC';
    const fragmentedResult = '349ABC';
    const fileMedia = new MockFileMedia(bufferString);

    const rarStream = new RarStream([
        new RarFileChunk(fileMedia, 1, 2),
        new RarFileChunk(fileMedia, 4, 6),
    ]);
    const buffer = await streamToBuffer(rarStream);
    t.deepEqual(buffer, new Buffer(fragmentedResult, 'hex'));
});

test('rar stream should stream over longer list of file chunks', async t => {
    const bufferString = '123456789ABC';
    const fileMedia = new MockFileMedia(bufferString);

    const rarStream = new RarStream([
        new RarFileChunk(fileMedia, 0, 2),
        new RarFileChunk(fileMedia, 2, 4),
        new RarFileChunk(fileMedia, 4, 6),
    ]);

    const buffer = await streamToBuffer(rarStream);
    t.deepEqual(buffer, new Buffer(bufferString, 'hex'));
});
