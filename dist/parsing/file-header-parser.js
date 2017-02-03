'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _binary = require('binary');

var _binary2 = _interopRequireDefault(_binary);

var _abstractParser = require('./abstract-parser');

var _abstractParser2 = _interopRequireDefault(_abstractParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class FileHeaderParser extends _abstractParser2.default {
    _parseFlags() {
        return parsedVars => {
            parsedVars.continuesFromPrevious = (parsedVars.flags & 0x01) !== 0;
            parsedVars.continuesInNext = (parsedVars.flags & 0x02) !== 0;
            parsedVars.isEncrypted = (parsedVars.flags & 0x04) !== 0;
            parsedVars.hasComment = (parsedVars.flags & 0x08) !== 0;
            parsedVars.hasInfoFromPrevious = (parsedVars.flags & 0x10) !== 0;
            parsedVars.hasHighSize = (parsedVars.flags & 0x100) !== 0;
            parsedVars.hasSpecialName = (parsedVars.flags & 0x200) !== 0;
            parsedVars.hasSalt = (parsedVars.flags & 0x400) !== 0;
            parsedVars.isOldVersion = (parsedVars.flags & 0x800) !== 0;
            parsedVars.hasExtendedTime = (parsedVars.flags & 0x1000) !== 0;
        };
    }
    _parseFileName() {
        return function (parsedVars) {
            let { vars: { nameBuffer } } = this.buffer('nameBuffer', parsedVars.nameSize);
            parsedVars.name = nameBuffer.toString('utf-8');
        };
    }
    _handleHighFileSize() {
        return function (parsedVars) {
            if (parsedVars.hasHighSize) {
                let { vars: { highPackSize, highUnpackSize } } = this.word32ls('highPackSize').word32ls('highUnpackSize');

                parsedVars.size = highPackSize * 0x100000000 + parsedVars.size;
                parsedVars.unpackedSize = highUnpackSize * 0x100000000 + parsedVars.unpackedSize;
            }
        };
    }

    get bytesToRead() {
        return FileHeaderParser.bytesToRead;
    }
    parse() {
        let { vars: fileHeader } = _binary2.default.parse(this.read()).word16lu('crc').word8lu('type').word16lu('flags').word16lu('headSize').word32lu('size').word32lu('unpackedSize').word8lu('host').word32lu('fileCrc').word32lu('timestamp').word8lu('version').word8lu('method').word16lu('nameSize').word32lu('attributes').tap(this._parseFlags()).tap(this._handleHighFileSize()).tap(this._parseFileName());
        return fileHeader;
    }
}
exports.default = FileHeaderParser;
Object.defineProperty(FileHeaderParser, 'bytesToRead', {
    enumerable: true,
    writable: true,
    value: 280
});