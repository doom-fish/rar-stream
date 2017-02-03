'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _rarFileBundle = require('../rar-file/rar-file-bundle');

var _rarFileBundle2 = _interopRequireDefault(_rarFileBundle);

var _rarFile = require('../rar-file/rar-file');

var _rarFile2 = _interopRequireDefault(_rarFile);

var _rarFileChunk = require('../rar-file/rar-file-chunk');

var _rarFileChunk2 = _interopRequireDefault(_rarFileChunk);

var _fileMedia = require('../file-media/file-media');

var _fileMedia2 = _interopRequireDefault(_fileMedia);

var _markerHeaderParser = require('../parsing/marker-header-parser');

var _markerHeaderParser2 = _interopRequireDefault(_markerHeaderParser);

var _archiveHeaderParser = require('../parsing/archive-header-parser');

var _archiveHeaderParser2 = _interopRequireDefault(_archiveHeaderParser);

var _fileHeaderParser = require('../parsing/file-header-parser');

var _fileHeaderParser2 = _interopRequireDefault(_fileHeaderParser);

var _terminatorHeaderParser = require('../parsing/terminator-header-parser');

var _terminatorHeaderParser2 = _interopRequireDefault(_terminatorHeaderParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

function _asyncToGenerator(fn) { return function () { var gen = fn.apply(this, arguments); return new Promise(function (resolve, reject) { function step(key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { return Promise.resolve(value).then(function (value) { step("next", value); }, function (err) { step("throw", err); }); } } return step("next"); }); }; }

class RarManifest {
    constructor(rarFileBundle) {
        this._rarFileBundle = rarFileBundle;
    }
    _parseMarkerHead(fileMedia) {
        return _asyncToGenerator(function* () {
            const interval = {
                start: 0,
                end: _markerHeaderParser2.default.bytesToRead
            };
            const stream = yield fileMedia.createReadStream(interval);
            const parser = new _markerHeaderParser2.default(stream);
            return parser.parse();
        })();
    }
    _parseArchiveHead(offset, fileMedia) {
        return _asyncToGenerator(function* () {
            const interval = {
                start: offset,
                end: _archiveHeaderParser2.default.bytesToRead
            };
            const stream = yield fileMedia.createReadStream(interval);
            const parser = new _archiveHeaderParser2.default(stream);
            return yield parser.parse();
        })();
    }
    _parseFileHead(offset, fileMedia) {
        return _asyncToGenerator(function* () {
            const interval = {
                start: offset,
                end: offset + _fileHeaderParser2.default.bytesToRead
            };

            const fileStream = yield fileMedia.createReadStream(interval);

            const parser = new _fileHeaderParser2.default(fileStream);
            return parser.parse();
        })();
    }
    _parse() {
        var _this = this;

        return _asyncToGenerator(function* () {
            const fileChunks = [];
            for (const rarFile of _this._rarFileBundle.files) {
                let fileOffset = 0;
                const markerHead = yield _this._parseMarkerHead(rarFile);
                fileOffset += markerHead.size;

                const archiveHeader = yield _this._parseArchiveHead(fileOffset, rarFile);
                fileOffset += archiveHeader.size;

                while (fileOffset < rarFile.size - _terminatorHeaderParser2.default.bytesToRead) {
                    const fileHead = yield _this._parseFileHead(fileOffset, rarFile);
                    if (fileHead.type !== 116) {
                        break;
                    }

                    fileOffset += fileHead.headSize;
                    fileChunks.push({
                        name: fileHead.name,
                        chunk: new _rarFileChunk2.default(rarFile, fileOffset, fileOffset + fileHead.size - 1)
                    });
                    fileOffset += fileHead.size;
                }
            }
            const grouped = fileChunks.reduce(function (file, { name, chunk }) {
                if (!file[name]) {
                    file[name] = [];
                }
                file[name].push(chunk);
                return file;
            }, {});

            return Object.keys(grouped).map(function (name) {
                return new _rarFile2.default(name, grouped[name]);
            });
        })();
    }
    getFiles() {
        return this._parse();
    }
}
exports.default = RarManifest;