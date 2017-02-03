'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.mockStreamFromString = exports.MockFileStream = undefined;

var _stream = require('stream');

class MockFileStream extends _stream.Readable {
    constructor(object, options) {
        super(options);
        this._options = options;
        this._object = object;
    }
    _read() {
        if (!!this._object && typeof this._options.start === 'number' && typeof this._options.end === 'number') {
            const buffer = this._object.slice(this._options.start, this._options.end);
            this.push(buffer);
            this._object = null;
        } else {
            this.push(this._object);
            this._object = null;
        }
    }
}

exports.MockFileStream = MockFileStream; // -disable

const mockStreamFromString = exports.mockStreamFromString = (str, options = {}, variant = 'hex') => {
    if (options.size) {
        let padding = Math.abs(+options.size - str.length / 2);
        str += Array(padding).fill().map(() => '00').join('');
    }
    return new MockFileStream(new Buffer(str, variant), options);
};