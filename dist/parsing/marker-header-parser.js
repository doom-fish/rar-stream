'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _binary = require('binary');

var _binary2 = _interopRequireDefault(_binary);

var _abstractParser = require('./abstract-parser');

var _abstractParser2 = _interopRequireDefault(_abstractParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class MarkerHeaderParser extends _abstractParser2.default {
    _addSizeIfFlagIsSet() {
        return function (parsedVars) {
            if ((parsedVars.flags & 0x8000) !== 0) {
                let { vars: { addSize } } = this.word32lu('addSize');
                parsedVars.size += addSize || 0;
            }
        };
    }
    get bytesToRead() {
        return MarkerHeaderParser.bytesToRead;
    }
    parse() {
        let { vars: markerHeader } = _binary2.default.parse(this.read()).word16lu('crc').word8lu('type').word16lu('flags').word16lu('size').tap(this._addSizeIfFlagIsSet());

        return markerHeader;
    }
}
exports.default = MarkerHeaderParser;
Object.defineProperty(MarkerHeaderParser, 'bytesToRead', {
    enumerable: true,
    writable: true,
    value: 11
});