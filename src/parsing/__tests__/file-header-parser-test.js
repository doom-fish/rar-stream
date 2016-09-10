//@flow
import test from 'ava';
import FileHeaderParser from '../file-header-parser';
import bind, { newPadding, hammingWeight, btoh } from '../__mocks__/utils';
const {newParser, parseHeader} = bind(FileHeaderParser, 280);

test('FileHeaderParser.bytesToRead should return 280', t => {
  t.is(newParser('00').bytesToRead, 280);
});

test('FileHeaderParser.parse should parse crc as 2 bytes', t => {
  t.is(hammingWeight(parseHeader('crc', 'ffff')), 16);
  t.is(hammingWeight(parseHeader('crc', 'fff0')), 12);
  t.is(hammingWeight(parseHeader('crc', 'ff00')), 8);
  t.is(hammingWeight(parseHeader('crc', 'f000')), 4);
  t.is(hammingWeight(parseHeader('crc', '0000')), 0);
});

test('FileHeaderParser.parse should parse crc as little endian', t => {
  t.is(parseHeader('crc', '1234'), 0x3412);
  t.is(parseHeader('crc', '3412'), 0x1234);
});

test('FileHeaderParser.parse should parse type as 1 byte', t => {
  const padding = newPadding(2);

  t.is(parseHeader('type', padding + 'ff'), 0xff);
  t.is(parseHeader('type', padding + '0f'), 0x0f);
  t.is(parseHeader('type', padding + 'ab'), 0xab);
  t.is(parseHeader('type', padding + 'ba'), 0xba);
});

test('FileHeaderParser.parse should parse flags as 2 bytes', t => {
  const padding = newPadding(3);
  t.is(hammingWeight(parseHeader('flags', padding + 'ffff')), 16);
  t.is(hammingWeight(parseHeader('flags', padding + 'fff0')), 12);
  t.is(hammingWeight(parseHeader('flags', padding + 'ff00')), 8);
  t.is(hammingWeight(parseHeader('flags', padding + 'f000')), 4);
  t.is(hammingWeight(parseHeader('flags', padding + '0000')), 0);
});

test('FileHeaderParser.parse should parse flags as little endian', t => {
  const padding = newPadding(3);
  t.is(parseHeader('flags', padding + '3412'), 0x1234);
  t.is(parseHeader('flags', padding + '1234'), 0x3412);
});

test('FileHeaderParser.parse should parse headSize as 2 bytes', t => {
  const padding = newPadding(5);
  t.is(hammingWeight(parseHeader('headSize', padding + 'ffff')), 16);
  t.is(hammingWeight(parseHeader('headSize', padding + 'fff0')), 12);
  t.is(hammingWeight(parseHeader('headSize', padding + 'ff00')), 8);
  t.is(hammingWeight(parseHeader('headSize', padding + 'f000')), 4);
  t.is(hammingWeight(parseHeader('headSize', padding + '0000')), 0);
});

test('FileHeaderParser.parse should parse headSize as little endian', t => {
  const padding = newPadding(5);
  t.is(parseHeader('headSize', padding + '3412'), 0x1234);
  t.is(parseHeader('headSize', padding + '1234'), 0x3412);
});

test('FileHeaderParser.parse should parse size as 4 bytes', t => {
  const padding = newPadding(7);
  t.is(hammingWeight(parseHeader('size', padding + 'ffffffff')), 32);
  t.is(hammingWeight(parseHeader('size', padding + 'ffffff00')), 24);
  t.is(hammingWeight(parseHeader('size', padding + 'ffff0000')), 16);
  t.is(hammingWeight(parseHeader('size', padding + 'ff000000')), 8);
  t.is(hammingWeight(parseHeader('size', padding + '00000000')), 0);
});

test('FileHeaderParser.parse should parse size as little endian', t => {
  const padding = newPadding(7);
  t.is(parseHeader('size', padding + '78563412'), 0x12345678);
  t.is(parseHeader('size', padding + '12345678'), 0x78563412);
});


test('FileHeaderParser.parse should parse unpackedSize as 4 bytes', t => {
  const padding = newPadding(11);
  t.is(hammingWeight(parseHeader('unpackedSize', padding + 'ffffffff')), 32);
  t.is(hammingWeight(parseHeader('unpackedSize', padding + 'ffffff00')), 24);
  t.is(hammingWeight(parseHeader('unpackedSize', padding + 'ffff0000')), 16);
  t.is(hammingWeight(parseHeader('unpackedSize', padding + 'ff000000')), 8);
  t.is(hammingWeight(parseHeader('unpackedSize', padding + '00000000')), 0);
});

test('FileHeaderParser.parse should parse unpackedSize as little endian', t => {
  const padding = newPadding(11);
  t.is(parseHeader('unpackedSize', padding + '78563412'), 0x12345678);
  t.is(parseHeader('unpackedSize', padding + '12345678'), 0x78563412);
});

test('FileHeaderParser.parse should parse host as 1 byte', t => {
  const padding = newPadding(15);
  t.is(parseHeader('host', padding + 'ff'), 0xff);
});

test('FileHeaderParser.parse should parse fileCrc as 4 bytes', t => {
  const padding = newPadding(16);
  t.is(hammingWeight(parseHeader('fileCrc', padding + 'ffffffff')), 32);
  t.is(hammingWeight(parseHeader('fileCrc', padding + 'ffffff00')), 24);
  t.is(hammingWeight(parseHeader('fileCrc', padding + 'ffff0000')), 16);
  t.is(hammingWeight(parseHeader('fileCrc', padding + 'ff000000')), 8);
  t.is(hammingWeight(parseHeader('fileCrc', padding + '00000000')), 0);
});

test('FileHeaderParser.parse should parse fileCrc as little endian', t => {
  const padding = newPadding(16);
  t.is(parseHeader('fileCrc', padding + '78563412'), 0x12345678);
  t.is(parseHeader('fileCrc', padding + '12345678'), 0x78563412);
});

test('FileHeaderParser.parse should parse timestamp as 4 bytes', t => {
  const padding = newPadding(20);
  t.is(hammingWeight(parseHeader('timestamp', padding + 'ffffffff')), 32);
  t.is(hammingWeight(parseHeader('timestamp', padding + 'ffffff00')), 24);
  t.is(hammingWeight(parseHeader('timestamp', padding + 'ffff0000')), 16);
  t.is(hammingWeight(parseHeader('timestamp', padding + 'ff000000')), 8);
  t.is(hammingWeight(parseHeader('timestamp', padding + '00000000')), 0);
});

test('FileHeaderParser.parse should parse timestamp as little endian', t => {
  const padding = newPadding(20);
  t.is(parseHeader('timestamp', padding + '78563412'), 0x12345678);
  t.is(parseHeader('timestamp', padding + '12345678'), 0x78563412);
});

test('FileHeaderParser.parse should parse version as 1 bytes', t => {
  const padding = newPadding(24);
  t.is(parseHeader('version', padding + 'ff'), 0xff);
  t.is(parseHeader('version', padding + 'ab'), 0xab);
  t.is(parseHeader('version', padding + 'dd'), 0xdd);
  t.is(parseHeader('version', padding + '00'), 0x0);
});

test('FileHeaderParser.parse should parse method as 1 bytes', t => {
  const padding = newPadding(25);
  t.is(parseHeader('method', padding + 'ff'), 0xff);
  t.is(parseHeader('method', padding + 'ab'), 0xab);
  t.is(parseHeader('method', padding + 'dd'), 0xdd);
  t.is(parseHeader('method', padding + '00'), 0x0);
});

test('FileHeaderParser.parse should parse nameSize as 2 bytes', t => {
  const padding = newPadding(26);
  t.is(hammingWeight(parseHeader('nameSize', padding + 'ffff')), 16);
  t.is(hammingWeight(parseHeader('nameSize', padding + 'fff0')), 12);
  t.is(hammingWeight(parseHeader('nameSize', padding + 'ff00')), 8);
  t.is(hammingWeight(parseHeader('nameSize', padding + 'f000')), 4);
  t.is(hammingWeight(parseHeader('nameSize', padding + '0000')), 0);
});

test('FileHeaderParser.parse should parse nameSize as little endian', t => {
  const padding = newPadding(26);
  t.is(parseHeader('nameSize', padding + '1234'), 0x3412);
  t.is(parseHeader('nameSize', padding + '3412'), 0x1234);
});


test('FileHeaderParser.parse should parse attributes as 2 bytes', t => {
  const padding = newPadding(28);
  t.is(hammingWeight(parseHeader('attributes', padding + 'ffffffff')), 32);
  t.is(hammingWeight(parseHeader('attributes', padding + 'ffffff00')), 24);
  t.is(hammingWeight(parseHeader('attributes', padding + 'ffff0000')), 16);
  t.is(hammingWeight(parseHeader('attributes', padding + 'ff000000')), 8);
  t.is(hammingWeight(parseHeader('attributes', padding + '00000000')), 0);
});

test('FileHeaderParser.parse should parse attributes as little endian', t => {
  const padding = newPadding(28);
  t.is(parseHeader('attributes', padding + '78563412'), 0x12345678);
  t.is(parseHeader('attributes', padding + '12345678'), 0x78563412);
});

test('FileHeaderParser.parse should parse continuesFromPrevious flag', t => {
  const bitField = 0b10000000000000000000000000000001;
  t.truthy(parseHeader('continuesFromPrevious', btoh(bitField)));
  t.falsy(parseHeader('continuesFromPrevious', '00'));
});

test('FileHeaderParser.parse should parse continuesInNext flag', t => {
  const bitField = 0b10000000000000000000000000000010;
  t.truthy(parseHeader('continuesInNext', btoh(bitField)));
  t.falsy(parseHeader('continuesInNext', '00'));
});

test('FileHeaderParser.parse should parse isEncrypted flag', t => {
  const bitField = 0b10000000000000000000000000000100;
  t.truthy(parseHeader('isEncrypted', btoh(bitField)));
  t.falsy(parseHeader('isEncrypted', '00'));
});

test('FileHeaderParser.parse should parse hasComment flag', t => {
  const bitField = 0b10000000000000000000000000001000;
  t.truthy(parseHeader('hasComment', btoh(bitField)));
  t.falsy(parseHeader('hasComment', '00'));
});

test('FileHeaderParser.parse should parse hasInfoFromPrevious flag', t => {
  const bitField = 0b10000000000000000000000000010000;
  t.truthy(parseHeader('hasInfoFromPrevious', btoh(bitField)));
  t.falsy(parseHeader('hasInfoFromPrevious', '00'));
});

test('FileHeaderParser.parse should parse hasHighSize flag', t => {
  const bitField = 0b1000000000000000000000000000000000000001;
  t.truthy(parseHeader('hasHighSize', btoh(bitField)));
  t.falsy(parseHeader('hasHighSize', '00'));
});

test('FileHeaderParser.parse should parse hasSpecialName flag', t => {
  const bitField = 0b1000000000000000000000000000000000000010;
  t.truthy(parseHeader('hasSpecialName', btoh(bitField)));
  t.falsy(parseHeader('hasSpecialName', '00'));
});

test('FileHeaderParser.parse should parse hasSalt flag', t => {
  const bitField = 0b1000000000000000000000000000000000000100;
  t.truthy(parseHeader('hasSalt', btoh(bitField)));
  t.falsy(parseHeader('hasSalt', '00'));
});

test('FileHeaderParser.parse should parse isOldVersion flag', t => {
  const bitField = 0b1000000000000000000000000000000000001000;
  t.truthy(parseHeader('isOldVersion', btoh(bitField)));
  t.falsy(parseHeader('isOldVersion', '00'));
});

test('FileHeaderParser.parse should parse hasExtendedTime flag', t => {
  const bitField = 0b1000000000000000000000000000000000010000;
  t.truthy(parseHeader('hasExtendedTime', btoh(bitField)));
  t.falsy(parseHeader('hasExtendedTime', '00'));
});

test('FileHeaderParser.parse should handle high file size', t => {
  const data = 'D97774111111115C1000005C10000003C5A6D2158A5' +
               '95B4714300A00A4810000040000000400000061636B6' +
               'E6F772E74787400C0';

  t.truthy(parseHeader('hasHighSize', data));
  t.is(parseHeader('size', data), 0x40000105c);
  t.is(parseHeader('unpackedSize', data), 0x40000105c);
});

test('FileHeaderParser.parse should parse name properly', t => {
  const data = 'D97774111111115C1000005C10000003C5A6D2158A5' +
               '95B4714300A00A4810000040000000400000061636B6' +
               'E6F772E74787400C0';
  t.is(parseHeader('name', data), 'acknow.txt');
});
