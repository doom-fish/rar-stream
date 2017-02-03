'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _terminatorHeaderParser = require('../terminator-header-parser');

var _terminatorHeaderParser2 = _interopRequireDefault(_terminatorHeaderParser);

var _utils = require('../__mocks__/utils');

var _utils2 = _interopRequireDefault(_utils);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

const { newParser, parseHeader } = (0, _utils2.default)(_terminatorHeaderParser2.default, 7);


(0, _ava2.default)('TerminatorHeaderParser.bytesToRead should be 7', t => {
    t.is(newParser('00').bytesToRead, 7);
});

(0, _ava2.default)('TerminatorHeaderParser.parse should parse 2 first bytes as crc', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ffffAB')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ffff00')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ffffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ffffAB')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ff0000')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'f00000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', '000000')), 0);
    t.is(parseHeader('crc', 'ffffAB'), 0xffff);
});

(0, _ava2.default)('TerminatorHeaderParser.parse should parse in little endian', t => {
    t.is(parseHeader('crc', '3412'), 0x1234);
    t.is(parseHeader('crc', '1234'), 0x3412);
});

(0, _ava2.default)('TerminatorHeaderParser.parse should parse type as 1 byte', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('type', 'FFFFFF')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('type', 'FFFF00')), 0);
    t.is(parseHeader('type', 'FFFFFF'), 0xff);
    t.is(parseHeader('type', 'FFFFFA'), 0xfa);
    t.is(parseHeader('type', 'FFFF0A'), 0x0a);
});

(0, _ava2.default)('TerminatorHeaderParser.parse should parse flags as 2 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffffffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffff0000')), 0);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffff000F')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffff00F0')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'ffffff0F00')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', 'fffffff000')), 4);

    t.is(parseHeader('flags', 'ffffff0000'), 0);
    t.is(parseHeader('flags', 'ffffffffff'), 0xffff);
});

(0, _ava2.default)('TerminatorHeaderParser.parse should parse flags as little endian', t => {
    t.is(parseHeader('flags', 'ffffff00FF'), 0xff00);
    t.is(parseHeader('flags', 'ffffffff00'), 0x00ff);
    t.is(parseHeader('flags', 'ffffff1234'), 0x3412);
    t.is(parseHeader('flags', 'ffffff3412'), 0x1234);
});

(0, _ava2.default)('TerminatorHeaderParser.parse should parse size as 2 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('size', 'ffffffffffffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('size', 'ffffffffff0000')), 0);
    t.is((0, _utils.hammingWeight)(parseHeader('size', 'ffffffffff000F')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('size', 'ffffffffff00F0')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('size', 'ffffffffff0F00')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('size', 'fffffffffff000')), 4);

    t.is(parseHeader('size', 'ffffffffff0000'), 0);
    t.is(parseHeader('size', 'ffffffffffffff'), 0xffff);
});

(0, _ava2.default)('TerminatorHeaderParser.parse should parse size as little endian', t => {
    t.is(parseHeader('size', 'ffffffffff00FF'), 0xff00);
    t.is(parseHeader('size', 'ffffffffffff00'), 0x00ff);
    t.is(parseHeader('size', 'ffffffffff1234'), 0x3412);
    t.is(parseHeader('size', 'ffffffffff3412'), 0x1234);
});