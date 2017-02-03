'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _markerHeaderParser = require('../marker-header-parser');

var _markerHeaderParser2 = _interopRequireDefault(_markerHeaderParser);

var _utils = require('../__mocks__/utils');

var _utils2 = _interopRequireDefault(_utils);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

const { newParser, parseHeader } = (0, _utils2.default)(_markerHeaderParser2.default, 11);


(0, _ava2.default)('MarkerHeaderParser.bytesToRead should be 11', t => {
    t.is(newParser('00').bytesToRead, 11);
});

(0, _ava2.default)('MarkerHeaderParser.parse should parse crc as 2 bytes', t => {
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'fff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'ff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', 'f000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('crc', '0000')), 0);
});

(0, _ava2.default)('MarkerHeaderParser.parse should parse crc as little endian', t => {
    t.is(parseHeader('crc', '1234'), 0x3412);
    t.is(parseHeader('crc', '3412'), 0x1234);
});

(0, _ava2.default)('MarkerHeaderParser.parse should parse type as 1 byte', t => {
    const padding = (0, _utils.newPadding)(2);
    t.is(parseHeader('type', padding + '72'), 0x72);
    t.is(parseHeader('type', padding + 'ff'), 0xff);
    t.is(parseHeader('type', padding + '01'), 0x01);
});

(0, _ava2.default)('MarkerHeaderParser.parse should parse flags as 2 bytes', t => {
    const padding = (0, _utils.newPadding)(3);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'ffff')), 16);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'fff0')), 12);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'ff00')), 8);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + 'f000')), 4);
    t.is((0, _utils.hammingWeight)(parseHeader('flags', padding + '0000')), 0);
});

(0, _ava2.default)('MarkerheaderParser.parse should parse flags as little endian', t => {
    const padding = (0, _utils.newPadding)(3);
    t.is(parseHeader('flags', padding + '1234'), 0x3412);
    t.is(parseHeader('flags', padding + '3412'), 0x1234);
});

(0, _ava2.default)('MarkerHeaderParser.parse should parse size as 1 byte', t => {
    const padding = (0, _utils.newPadding)(5);
    t.is(parseHeader('size', padding + 'ff'), 0xff);
    t.is(parseHeader('size', padding + 'ab'), 0xab);
    t.is(parseHeader('size', padding + 'f0'), 0xf0);
    t.is(parseHeader('size', padding + 'f1'), 0xf1);
});

(0, _ava2.default)('MarkerHeaderParser.parse should parse add_size flag', t => {
    t.is(parseHeader('size', '526172219A070001000000'), 0x08);
    t.is(parseHeader('size', '526172219A070009000000'), 0x10);
    t.is(parseHeader('size', '526172219A07000A000000'), 0x11);
    t.is(parseHeader('size', '526172219A0700F8FFFFFF'), 0xffffffff);
});