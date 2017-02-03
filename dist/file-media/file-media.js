'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _stream = require('stream');

class FileMedia {
    constructor(fileInfo) {
        this._createReadStream = interval => fileInfo.createReadStream(interval);
        this._name = fileInfo.name;
        this._size = fileInfo.size;
    }
    get name() {
        return this._name;
    }
    get size() {
        return this._size;
    }

    createReadStreamSync(interval) {
        const { start, end } = interval;
        if (start > end) {
            throw Error('Invalid Arguments, start offset can not be greater than end offset');
        }
        return this._createReadStream({ start, end: end });
    }

    createReadStream(interval) {
        const { start, end } = interval;
        if (start > end) {
            throw Error('Invalid Arguments, start offset can not be greater than end offset');
        }
        let stream = this._createReadStream({ start, end: end });

        return new Promise((resolve, reject) => {
            stream.on('readable', () => resolve(stream));
            stream.on('error', error => reject(error));
        });
    }
}
exports.default = FileMedia;