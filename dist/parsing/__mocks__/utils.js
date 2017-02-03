'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.newPadding = newPadding;
exports.hammingWeight = hammingWeight;
exports.btoh = btoh;

var _mockBufferStream = require('./mock-buffer-stream');

var _abstractParser = require('../abstract-parser');

var _abstractParser2 = _interopRequireDefault(_abstractParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

function newPadding(count) {
    return Array(count * 2).fill('0').join('');
}

function hammingWeight(num) {
    num = num - (num >> 1 & 0x55555555);
    num = (num & 0x33333333) + (num >> 2 & 0x33333333);
    return (num + (num >> 4) & 0xf0f0f0f) * 0x1010101 >> 24;
}

exports.default = (Parser, size) => ({
    newParser(binaryStr) {
        return new Parser((0, _mockBufferStream.mockStreamFromString)(binaryStr, { size: size }));
    },
    parseHeader(field, binaryStr) {
        return new Parser((0, _mockBufferStream.mockStreamFromString)(binaryStr, { size: size })).parse()[field];
    }
});

function btoh(binary) {
    const str = binary.toString(16);
    return str.length % 2 !== 0 ? '0' + str : str;
}