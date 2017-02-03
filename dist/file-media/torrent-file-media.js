'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _fileMedia = require('./file-media');

var _fileMedia2 = _interopRequireDefault(_fileMedia);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class TorrentFileMedia extends _fileMedia2.default {
    constructor(torrentFileInfo) {
        torrentFileInfo.select();
        torrentFileInfo.size = torrentFileInfo.length;
        super(torrentFileInfo);
    }
}
exports.default = TorrentFileMedia;