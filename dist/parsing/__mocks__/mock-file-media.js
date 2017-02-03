'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _mockBufferStream = require('./mock-buffer-stream');

var _fileMedia = require('../../file-media/file-media');

var _fileMedia2 = _interopRequireDefault(_fileMedia);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

class MockFileMedia extends _fileMedia2.default {
    constructor(stringData, name) {
        stringData = stringData.replace(/\s/g, '');
        const byteLength = stringData.length;
        super({
            name: name || 'MockStream',
            size: byteLength / 2,
            createReadStream: ({ start, end }) => {
                return (0, _mockBufferStream.mockStreamFromString)(stringData, {
                    start,
                    end,
                    byteLength
                });
            }
        });
    }
}
exports.default = MockFileMedia;