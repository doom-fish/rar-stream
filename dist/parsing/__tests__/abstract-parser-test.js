'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _mockBufferStream = require('../__mocks__/mock-buffer-stream');

var _mockAbstractParser = require('../__mocks__/mock-abstract-parser');

var _mockAbstractParser2 = _interopRequireDefault(_mockAbstractParser);

var _abstractParser = require('../abstract-parser');

var _abstractParser2 = _interopRequireDefault(_abstractParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

function newMock(bufferStr, size, options = {}) {
    const stream = (0, _mockBufferStream.mockStreamFromString)(bufferStr, options);
    return new _mockAbstractParser2.default(stream, size);
}

function newParser(bufferStr = '00') {
    return new _abstractParser2.default((0, _mockBufferStream.mockStreamFromString)(bufferStr));
}

(0, _ava2.default)('AbstractParser should be constructable', t => {
    t.truthy(newParser() instanceof _abstractParser2.default);
});

(0, _ava2.default)('AbstractParser.read() should read from a stream and return a buffer', t => {
    let mock = newMock('AF', 1);
    const withSizeInstanceResult = mock.read(1);

    t.is(withSizeInstanceResult && withSizeInstanceResult.length, 1);
    t.deepEqual(withSizeInstanceResult, new Buffer('AF', 'hex'));

    mock = newMock('0123456789ABCDEF', 8);
    let withBiggerBufferResult = mock.read(8);
    t.is(withBiggerBufferResult && withBiggerBufferResult.length, 8);
    t.deepEqual(withBiggerBufferResult, new Buffer('0123456789ABCDEF', 'hex'));
});