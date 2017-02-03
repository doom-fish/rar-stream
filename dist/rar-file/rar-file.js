'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _rarStream = require('./rar-stream');

var _rarStream2 = _interopRequireDefault(_rarStream);

var _rarFileChunk = require('./rar-file-chunk');

var _rarFileChunk2 = _interopRequireDefault(_rarFileChunk);

var _streamToBuffer = require('stream-to-buffer');

var _streamToBuffer2 = _interopRequireDefault(_streamToBuffer);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class RarFile {
    constructor(name, rarFileChunks) {
        this._rarFileChunks = rarFileChunks;
        this._chunkMap = this._calculateChunkMap(rarFileChunks);
        this._size = this._rarFileChunks.reduce((size, chunk) => size + chunk.length, 0);
        this._name = name;
    }
    readToEnd() {
        return new Promise((resolve, reject) => {
            (0, _streamToBuffer2.default)(this.createReadStream({ start: 0, end: this._size }), (err, buffer) => {
                if (err) {
                    reject(err);
                } else {
                    resolve(buffer);
                }
            });
        });
    }
    getChunksToStream(start, end) {
        const {
            index: startIndex,
            start: startOffset
        } = this._findMappedChunk(start);

        const {
            index: endIndex,
            end: endOffset
        } = this._findMappedChunk(end);

        const chunksToStream = this._rarFileChunks.slice(startIndex, endIndex + 1);

        const last = chunksToStream.length - 1;
        chunksToStream[0] = chunksToStream[0].paddStart(Math.abs(startOffset - start));

        chunksToStream[last] = chunksToStream[last].paddEnd(Math.abs(endOffset - end));

        return chunksToStream;
    }
    createReadStream(interval) {
        if (!interval) {
            interval = { start: 0, end: this._size };
        }
        const { start, end } = interval;

        if (start < 0 || end > this._size) {
            throw Error('Illegal start/end offset');
        }

        return new _rarStream2.default(this.getChunksToStream(start, end));
    }
    get name() {
        return this._name;
    }
    get size() {
        return this._size;
    }
    _calculateChunkMap(rarFileChunks) {
        const chunkMap = [];
        let index = 0;
        for (const chunk of rarFileChunks) {
            const previousChunk = chunkMap[chunkMap.length - 1];
            const start = previousChunk && previousChunk.end || 0;
            const end = start + chunk.length;
            chunkMap.push({ index, start, end, chunk });
            index++;
        }

        return chunkMap;
    }
    _findMappedChunk(offset) {
        let selectedMap = this._chunkMap[0];
        for (const map of this._chunkMap) {
            if (offset >= map.start && offset <= map.end) {
                selectedMap = map;
                break;
            }
        }
        return selectedMap;
    }
}
exports.default = RarFile;