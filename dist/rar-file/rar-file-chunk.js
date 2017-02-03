'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _stream = require('stream');

var _fileMedia = require('../file-media/file-media');

var _fileMedia2 = _interopRequireDefault(_fileMedia);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class RarFileChunk {
    constructor(fileMedia, startOffset, endOffset) {
        this._fileMedia = fileMedia;
        this._startOffset = startOffset;
        this._endOffset = endOffset;
    }
    paddEnd(endPadding) {
        return new RarFileChunk(this._fileMedia, this._startOffset, this._endOffset - endPadding);
    }
    paddStart(startPadding) {
        return new RarFileChunk(this._fileMedia, this._startOffset + startPadding, this._endOffset);
    }
    set startOffset(value) {
        this._startOffset = value;
    }
    set endOffset(value) {
        this._endOffset = value;
    }
    get length() {
        return Math.abs(this._endOffset - this._startOffset);
    }
    getStreamSync() {
        return this._fileMedia.createReadStreamSync({
            start: this._startOffset,
            end: this._endOffset
        });
    }
    getStream() {
        return this._fileMedia.createReadStream({
            start: this._startOffset,
            end: this._endOffset
        });
    }
}
exports.default = RarFileChunk;