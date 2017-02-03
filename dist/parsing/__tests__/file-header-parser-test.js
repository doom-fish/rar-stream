'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _fileHeaderParser = require('../file-header-parser');

var _fileHeaderParser2 = _interopRequireDefault(_fileHeaderParser);

var _utils = require('../__mocks__/utils');

var _utils2 = _interopRequireDefault(_utils);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

const { newParser, parseHeader } = (0, _utils2.default)(_fileHeaderParser2.default, 280);


(0, _ava2.default)('FileHeaderParser.bytesToRead should return 280', t => {
    t.is(newParser('00').bytesToRead, 280);
});

(0, _ava2.default)('FileHeaderParser.parse should parse crc as 2 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'fff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'f000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', '0000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse crc as little endian', t => {
    t.is(parseHeader('crc', '1234'), 0x3412);
    t.is(parseHeader('crc', '3412'), 0x1234);
});

(0, _ava2.default)('FileHeaderParser.parse should parse type as 1 byte', t => {
    const padding = (0, _utils.newPadding)(2);

    t.is(parseHeader('type', padding + 'ff'), 0xff);
    t.is(parseHeader('type', padding + '0f'), 0x0f);
    t.is(parseHeader('type', padding + 'ab'), 0xab);
    t.is(parseHeader('type', padding + 'ba'), 0xba);
});

(0, _ava2.default)('FileHeaderParser.parse should parse flags as 2 bytes', t => {
    const padding = (0, _utils.newPadding)(3);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'fff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'ff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'f000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + '0000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse flags as little endian', t => {
    const padding = (0, _utils.newPadding)(3);
    t.is(parseHeader('flags', padding + '3412'), 0x1234);
    t.is(parseHeader('flags', padding + '1234'), 0x3412);
});

(0, _ava2.default)('FileHeaderParser.parse should parse headSize as 2 bytes', t => {
    const padding = (0, _utils.newPadding)(5);
    t.is((0, _utils.hammingWeight)(parseHeader('headSize', padding + 'ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('headSize', padding + 'fff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('headSize', padding + 'ff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('headSize', padding + 'f000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('headSize', padding + '0000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse headSize as little endian', t => {
    const padding = (0, _utils.newPadding)(5);
    t.is(parseHeader('headSize', padding + '3412'), 0x1234);
    t.is(parseHeader('headSize', padding + '1234'), 0x3412);
});

(0, _ava2.default)('FileHeaderParser.parse should parse size as 4 bytes', t => {
    const padding = (0, _utils.newPadding)(7);
    t.is((0, _utils.hammingWeight)(parseHeader('size', padding + 'ffffffff')), 32);
    t.is((0, _utils.hammingWeight)(parseHeader('size', padding + 'ffffff00')), 24);
    t.is((0, _utils.hammingWeight)(parseHeader('size', padding + 'ffff0000')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('size', padding + 'ff000000')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('size', padding + '00000000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse size as little endian', t => {
    const padding = (0, _utils.newPadding)(7);
    t.is(parseHeader('size', padding + '78563412'), 0x12345678);
    t.is(parseHeader('size', padding + '12345678'), 0x78563412);
});

(0, _ava2.default)('FileHeaderParser.parse should parse unpackedSize as 4 bytes', t => {
    const padding = (0, _utils.newPadding)(11);
    t.is((0, _utils.hammingWeight)(parseHeader('unpackedSize', padding + 'ffffffff')), 32);
    t.is((0, _utils.hammingWeight)(parseHeader('unpackedSize', padding + 'ffffff00')), 24);
    t.is((0, _utils.hammingWeight)(parseHeader('unpackedSize', padding + 'ffff0000')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('unpackedSize', padding + 'ff000000')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('unpackedSize', padding + '00000000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse unpackedSize as little endian', t => {
    const padding = (0, _utils.newPadding)(11);
    t.is(parseHeader('unpackedSize', padding + '78563412'), 0x12345678);
    t.is(parseHeader('unpackedSize', padding + '12345678'), 0x78563412);
});

(0, _ava2.default)('FileHeaderParser.parse should parse host as 1 byte', t => {
    const padding = (0, _utils.newPadding)(15);
    t.is(parseHeader('host', padding + 'ff'), 0xff);
});

(0, _ava2.default)('FileHeaderParser.parse should parse fileCrc as 4 bytes', t => {
    const padding = (0, _utils.newPadding)(16);
    t.is((0, _utils.hammingWeight)(parseHeader('fileCrc', padding + 'ffffffff')), 32);
    t.is((0, _utils.hammingWeight)(parseHeader('fileCrc', padding + 'ffffff00')), 24);
    t.is((0, _utils.hammingWeight)(parseHeader('fileCrc', padding + 'ffff0000')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('fileCrc', padding + 'ff000000')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('fileCrc', padding + '00000000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse fileCrc as little endian', t => {
    const padding = (0, _utils.newPadding)(16);
    t.is(parseHeader('fileCrc', padding + '78563412'), 0x12345678);
    t.is(parseHeader('fileCrc', padding + '12345678'), 0x78563412);
});

(0, _ava2.default)('FileHeaderParser.parse should parse timestamp as 4 bytes', t => {
    const padding = (0, _utils.newPadding)(20);
    t.is((0, _utils.hammingWeight)(parseHeader('timestamp', padding + 'ffffffff')), 32);
    t.is((0, _utils.hammingWeight)(parseHeader('timestamp', padding + 'ffffff00')), 24);
    t.is((0, _utils.hammingWeight)(parseHeader('timestamp', padding + 'ffff0000')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('timestamp', padding + 'ff000000')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('timestamp', padding + '00000000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse timestamp as little endian', t => {
    const padding = (0, _utils.newPadding)(20);
    t.is(parseHeader('timestamp', padding + '78563412'), 0x12345678);
    t.is(parseHeader('timestamp', padding + '12345678'), 0x78563412);
});

(0, _ava2.default)('FileHeaderParser.parse should parse version as 1 bytes', t => {
    const padding = (0, _utils.newPadding)(24);
    t.is(parseHeader('version', padding + 'ff'), 0xff);
    t.is(parseHeader('version', padding + 'ab'), 0xab);
    t.is(parseHeader('version', padding + 'dd'), 0xdd);
    t.is(parseHeader('version', padding + '00'), 0x0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse method as 1 bytes', t => {
    const padding = (0, _utils.newPadding)(25);
    t.is(parseHeader('method', padding + 'ff'), 0xff);
    t.is(parseHeader('method', padding + 'ab'), 0xab);
    t.is(parseHeader('method', padding + 'dd'), 0xdd);
    t.is(parseHeader('method', padding + '00'), 0x0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse nameSize as 2 bytes', t => {
    const padding = (0, _utils.newPadding)(26);
    t.is((0, _utils.hammingWeight)(parseHeader('nameSize', padding + 'ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('nameSize', padding + 'fff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('nameSize', padding + 'ff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('nameSize', padding + 'f000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('nameSize', padding + '0000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse nameSize as little endian', t => {
    const padding = (0, _utils.newPadding)(26);
    t.is(parseHeader('nameSize', padding + '1234'), 0x3412);
    t.is(parseHeader('nameSize', padding + '3412'), 0x1234);
});

(0, _ava2.default)('FileHeaderParser.parse should parse attributes as 2 bytes', t => {
    const padding = (0, _utils.newPadding)(28);
    t.is((0, _utils.hammingWeight)(parseHeader('attributes', padding + 'ffffffff')), 32);
    t.is((0, _utils.hammingWeight)(parseHeader('attributes', padding + 'ffffff00')), 24);
    t.is((0, _utils.hammingWeight)(parseHeader('attributes', padding + 'ffff0000')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('attributes', padding + 'ff000000')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('attributes', padding + '00000000')), 0);
});

(0, _ava2.default)('FileHeaderParser.parse should parse attributes as little endian', t => {
    const padding = (0, _utils.newPadding)(28);
    t.is(parseHeader('attributes', padding + '78563412'), 0x12345678);
    t.is(parseHeader('attributes', padding + '12345678'), 0x78563412);
});

(0, _ava2.default)('FileHeaderParser.parse should parse continuesFromPrevious flag', t => {
    const bitField = 0b10000000000000000000000000000001;
    t.truthy(parseHeader('continuesFromPrevious', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('continuesFromPrevious', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse continuesInNext flag', t => {
    const bitField = 0b10000000000000000000000000000010;
    t.truthy(parseHeader('continuesInNext', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('continuesInNext', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse isEncrypted flag', t => {
    const bitField = 0b10000000000000000000000000000100;
    t.truthy(parseHeader('isEncrypted', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('isEncrypted', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse hasComment flag', t => {
    const bitField = 0b10000000000000000000000000001000;
    t.truthy(parseHeader('hasComment', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('hasComment', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse hasInfoFromPrevious flag', t => {
    const bitField = 0b10000000000000000000000000010000;
    t.truthy(parseHeader('hasInfoFromPrevious', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('hasInfoFromPrevious', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse hasHighSize flag', t => {
    const bitField = 0b1000000000000000000000000000000000000001;
    t.truthy(parseHeader('hasHighSize', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('hasHighSize', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse hasSpecialName flag', t => {
    const bitField = 0b1000000000000000000000000000000000000010;
    t.truthy(parseHeader('hasSpecialName', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('hasSpecialName', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse hasSalt flag', t => {
    const bitField = 0b1000000000000000000000000000000000000100;
    t.truthy(parseHeader('hasSalt', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('hasSalt', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse isOldVersion flag', t => {
    const bitField = 0b1000000000000000000000000000000000001000;
    t.truthy(parseHeader('isOldVersion', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('isOldVersion', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should parse hasExtendedTime flag', t => {
    const bitField = 0b1000000000000000000000000000000000010000;
    t.truthy(parseHeader('hasExtendedTime', (0, _utils.btoh)(bitField)));
    t.falsy(parseHeader('hasExtendedTime', '00'));
});

(0, _ava2.default)('FileHeaderParser.parse should handle high file size', t => {
    const data = 'D97774111111115C1000005C10000003C5A6D2158A5' + '95B4714300A00A4810000040000000400000061636B6' + 'E6F772E74787400C0';

    t.truthy(parseHeader('hasHighSize', data));
    t.is(parseHeader('size', data), 0x40000105c);
    t.is(parseHeader('unpackedSize', data), 0x40000105c);
});

(0, _ava2.default)('FileHeaderParser.parse should parse name properly', t => {
    const data = 'D97774111111115C1000005C10000003C5A6D2158A5' + '95B4714300A00A4810000040000000400000061636B6' + 'E6F772E74787400C0';
    t.is(parseHeader('name', data), 'acknow.txt');
});