'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _archiveHeaderParser = require('../archive-header-parser');

var _archiveHeaderParser2 = _interopRequireDefault(_archiveHeaderParser);

var _utils = require('../__mocks__/utils');

var _utils2 = _interopRequireDefault(_utils);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

const { newParser, parseHeader } = (0, _utils2.default)(_archiveHeaderParser2.default, 13);


(0, _ava2.default)('ArchiveHeaderParser.bytesToRead', t => {
    const parser = newParser('00');
    t.is(parser.bytesToRead, 13);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse CRC as 2 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ffff00ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ff0000ffff')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'f00000ffff')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', '000000ffff')), 0);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse CRC as little endian', t => {
    t.is(parseHeader('crc', '1234'), 0x3412);
    t.is(parseHeader('crc', '3412'), 0x1234);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse type as 1 byte', t => {
    t.is(parseHeader('crc', 'FF'), 0xff);
    t.is(parseHeader('crc', 'AB'), 0xab);
    t.is(parseHeader('crc', '00'), 0x00);
    t.is(parseHeader('crc', '0F'), 0x0f);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse flags as 2 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffff0000')), 0);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'fffffff000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffffff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'fffffffff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffffffff')), 16);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse flags as little endian', t => {
    t.is(parseHeader('flags', 'ffffff1234'), 0x3412);
    t.is(parseHeader('flags', 'ffffff3412'), 0x1234);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse size as 1 byte', t => {
    t.is(parseHeader('size', 'ffffffff00ff'), 0xff);
    t.is(parseHeader('size', 'ffffffff000f'), 0x0f);
    t.is(parseHeader('size', 'ffffffff0000'), 0x0d);
    t.is(parseHeader('size', 'ffffffff00AB'), 0xab);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse reserved1 as 2 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('reserved1', 'ffffffffffff00ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved1', 'ffffffffffff00fff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved1', 'ffffffffffff00ff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved1', 'ffffffffffff00f000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved1', 'ffffffffffffff0000')), 0);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse reserved1 as little endian', t => {
    t.is(parseHeader('reserved1', 'ffffffffffffff1234'), 0x3412);
    t.is(parseHeader('reserved1', 'ffffffffffffff3412'), 0x1234);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse reserved2 as 4 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('reserved2', 'ffffffffffffffff00ffffffffff')), 32);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved2', 'ffffffffffffffff00ffffff00ff')), 24);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved2', 'ffffffffffffffff00ffff0000ff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved2', 'ffffffffffffffff00ff000000ff')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('reserved2', 'ffffffffffffffff0000000000ff')), 0);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse reserved2 as little endian', t => {
    t.is(parseHeader('reserved2', 'ffffffffffffffffff12345678'), 0x78563412);
    t.is(parseHeader('reserved2', 'ffffffffffffffffff78563412'), 0x12345678);
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse hasVolumeAttributes flag', t => {
    t.truthy(parseHeader('hasVolumeAttributes', (0, _utils.btoh)(0b100000000000000000000000001)));
    t.falsy(parseHeader('hasVolumeAttributes', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse hasComment flag', t => {
    t.truthy(parseHeader('hasComment', (0, _utils.btoh)(0b100000000000000000000000010)));
    t.falsy(parseHeader('hasComment', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse isLocked flag', t => {
    t.truthy(parseHeader('isLocked', (0, _utils.btoh)(0b100000000000000000000000100)));
    t.falsy(parseHeader('isLocked', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse hasSolidAttributes flag', t => {
    t.truthy(parseHeader('hasSolidAttributes', (0, _utils.btoh)(0b100000000000000000000001000)));
    t.falsy(parseHeader('hasSolidAttributes', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse isNewNameScheme flag', t => {
    t.truthy(parseHeader('isNewNameScheme', (0, _utils.btoh)(0b100000000000000000000010000)));
    t.falsy(parseHeader('isNewNameScheme', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse hasAuthInfo flag', t => {
    t.truthy(parseHeader('hasAuthInfo', (0, _utils.btoh)(0b100000000000000000000100000)));
    t.falsy(parseHeader('hasAuthInfo', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse hasRecovery flag', t => {
    t.truthy(parseHeader('hasRecovery', (0, _utils.btoh)(0b100000000000000000001000000)));
    t.falsy(parseHeader('hasRecovery', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse isBlockEncoded flag', t => {
    t.truthy(parseHeader('isBlockEncoded', (0, _utils.btoh)(0b100000000000000000010000000)));
    t.falsy(parseHeader('isBlockEncoded', (0, _utils.btoh)(0b100000000000000000000000000)));
});

(0, _ava2.default)('ArchiveHeaderParser.parse should parse isFirstVolume flag', t => {
    t.truthy(parseHeader('isFirstVolume', (0, _utils.btoh)(0b100000000000000000000000000000001)));
    t.falsy(parseHeader('isFirstVolume', (0, _utils.btoh)(0b100000000000000000000000000000000)));
});