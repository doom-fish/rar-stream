// @flow
import test from 'ava';
import { mockStreamFromString } from '../__mocks__/mock-buffer-stream';
import MockAbstractParser from '../__mocks__/mock-abstract-parser';
import AbstractParser from '../abstract-parser';
type Options = readableStreamOptions;
function newMock(
    bufferStr: string,
    size: number,
    options: Options = {}
): MockAbstractParser {
    const stream = mockStreamFromString(bufferStr, options);
    return new MockAbstractParser(stream, size);
}

function newParser(bufferStr: string = '00'): AbstractParser {
    return new AbstractParser(mockStreamFromString(bufferStr));
}

test('AbstractParser should be constructable', t => {
    t.truthy(newParser() instanceof AbstractParser);
});

test(
    'AbstractParser.read() should read from a stream and return a buffer',
    t => {
        let mock = newMock('AF', 1);
        const withSizeInstanceResult = mock.read(1);

        t.is(withSizeInstanceResult && withSizeInstanceResult.length, 1);
        t.deepEqual(withSizeInstanceResult, new Buffer('AF', 'hex'));

        mock = newMock('0123456789ABCDEF', 8);
        let withBiggerBufferResult = mock.read(8);
        t.is(withBiggerBufferResult && withBiggerBufferResult.length, 8);
        t.deepEqual(withBiggerBufferResult, new Buffer(
            '0123456789ABCDEF',
            'hex'
        ));
    }
);
