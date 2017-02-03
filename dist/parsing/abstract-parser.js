'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _stream = require('stream');

class AbstractParser {
    constructor(stream) {
        this._stream = stream;
    }
    get bytesToRead() {
        throw Error('Abstract Getter, implement in sub classes');
    }
    parse() {
        throw Error('Abstract Method, implement in sub classes');
    }
    read() {
        return this._stream.read(this.bytesToRead);
    }
}
exports.default = AbstractParser;