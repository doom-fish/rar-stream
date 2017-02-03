'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _fileMedia = require('./file-media');

var _fileMedia2 = _interopRequireDefault(_fileMedia);

var _fs = require('fs');

var _fs2 = _interopRequireDefault(_fs);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class LocalFileMedia extends _fileMedia2.default {
    constructor(localFilePath) {
        if (typeof localFilePath !== 'string') {
            throw new Error('Invalid Arguments, localFilePath' + 'need to be passed to the constructor as a string');
        }
        let nameParts = localFilePath.split('/');
        let fileInfo = {
            name: nameParts[nameParts.length - 1],
            size: _fs2.default.statSync(localFilePath).size,
            createReadStream: options => _fs2.default.createReadStream(localFilePath, options)
        };
        super(fileInfo);
    }
}
exports.default = LocalFileMedia;