'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _stream = require('stream');

var _rarFileChunk = require('./rar-file-chunk');

var _rarFileChunk2 = _interopRequireDefault(_rarFileChunk);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class RarStream extends _stream.Readable {
    constructor(rarFileChunks) {
        super();
        this._rarFileChunks = rarFileChunks;
        this._next();
    }
    pushData(data) {
        if (!this.push(data)) {
            this._stream.pause();
        }
    }
    _next() {
        const chunk = this._rarFileChunks.shift();

        if (!chunk) {
            this.push(null);
        } else {
            this._stream = chunk.getStreamSync();
            this._stream.on('data', data => this.pushData(data));
            const self = this;
            this._stream.on('end', function () {
                self._next();
            });
        }
    }
    _read() {
        this._stream.resume();
    }
}
exports.default = RarStream;