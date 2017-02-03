'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _stream = require('stream');

var _abstractParser = require('../abstract-parser');

var _abstractParser2 = _interopRequireDefault(_abstractParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class MockAbstractParser extends _abstractParser2.default {
    constructor(stream, size) {
        super(stream);
        this._size = size;
    }
    get bytesToRead() {
        return this._size;
    }
}
exports.default = MockAbstractParser;