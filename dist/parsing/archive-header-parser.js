'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _binary = require('binary');

var _binary2 = _interopRequireDefault(_binary);

var _abstractParser = require('./abstract-parser');

var _abstractParser2 = _interopRequireDefault(_abstractParser);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class ArchiveHeaderParser extends _abstractParser2.default {
    _parseFlags() {
        return parsedVars => {
            parsedVars.hasVolumeAttributes = (parsedVars.flags & 0x0001) !== 0;
            parsedVars.hasComment = (parsedVars.flags & 0x0002) !== 0;
            parsedVars.isLocked = (parsedVars.flags & 0x0004) !== 0;
            parsedVars.hasSolidAttributes = (parsedVars.flags & 0x0008) !== 0;
            parsedVars.isNewNameScheme = (parsedVars.flags & 0x00010) !== 0;
            parsedVars.hasAuthInfo = (parsedVars.flags & 0x0020) !== 0;
            parsedVars.hasRecovery = (parsedVars.flags & 0x0040) !== 0;
            parsedVars.isBlockEncoded = (parsedVars.flags & 0x0080) !== 0;
            parsedVars.isFirstVolume = (parsedVars.flags & 0x0100) !== 0;
        };
    }
    get bytesToRead() {
        return ArchiveHeaderParser.bytesToRead;
    }
    parse() {
        let { vars: archiveHeader } = _binary2.default.parse(this.read()).word16lu('crc').word8lu('type').word16lu('flags').word16lu('size').word16lu('reserved1').word32lu('reserved2').tap(this._parseFlags());
        archiveHeader.size = archiveHeader.size || this.bytesToRead;
        return archiveHeader;
    }
}
exports.default = ArchiveHeaderParser;
Object.defineProperty(ArchiveHeaderParser, 'bytesToRead', {
    enumerable: true,
    writable: true,
    value: 13
});