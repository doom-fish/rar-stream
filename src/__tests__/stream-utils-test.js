const test = require('ava');
const { Duplex } = require('stream');
const { streamToBuffer, bufferToStream } = require('../stream-utils');

test('streamToBuffer is a function', t =>
  t.true(
    typeof streamToBuffer === 'function',
    'streamToBuffer is not a function'
  ));

test('bufferToStream is a function', t =>
  t.true(typeof bufferToStream === 'function', 'bufferToStream is a function'));

test('bufferToStream returns a Readable stream', t =>
  t.true(
    bufferToStream() instanceof Duplex,
    'bufferToStream does not return a stream'
  ));

test('stream to buffer conversion works both ways', async t => {
  const bufferContent = 'bufferString1234';
  const buffer = Buffer.from(bufferContent);
  const stream = bufferToStream(buffer);
  const readBuffer = await streamToBuffer(stream);
  t.deepEqual(buffer, readBuffer);
  t.is(bufferContent, readBuffer.toString('utf-8'));
});
