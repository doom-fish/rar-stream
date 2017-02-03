'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _binary = require('binary');

var _binary2 = _interopRequireDefault(_binary);

var _abstractParser = require('./abstract-parser');

var _abstractParser2 = _interopRequireDefault(_abstractParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class TerminatorHeaderParser extends _abstractParser2.default {
    get bytesToRead() {
        return TerminatorHeaderParser.bytesToRead;
    }
    parse() {
        let { vars: terminatorHeader } = _binary2.default.parse(this.read()).word16lu('crc').word8lu('type').word16lu('flags').word16lu('size');

        return terminatorHeader;
    }
}
exports.default = TerminatorHeaderParser;
Object.defineProperty(TerminatorHeaderParser, 'bytesToRead', {
    enumerable: true,
    writable: true,
    value: 7
});
Object.defineProperty(TerminatorHeaderParser, 'endOfArchivePadding', {
    enumerable: true,
    writable: true,
    value: 20
});