// @flow
import test from 'ava'
import ArchiveHeaderParser from '../archive-header-parser'
import bind, { hammingWeight, btoh } from '../__mocks__/utils'
const {newParser, parseHeader} = bind(ArchiveHeaderParser, 13)

test('ArchiveHeaderParser.bytesToRead', (t) => {
  const parser = newParser('00')
  t.is(parser.bytesToRead, 13)
})

test('ArchiveHeaderParser.parse should parse CRC as 2 bytes', (t) => {
  t.is(hammingWeight(parseHeader('crc', 'ffff00ffff')), 16)
  t.is(hammingWeight(parseHeader('crc', 'ff0000ffff')), 8)
  t.is(hammingWeight(parseHeader('crc', 'f00000ffff')), 4)
  t.is(hammingWeight(parseHeader('crc', '000000ffff')), 0)
})

test('ArchiveHeaderParser.parse should parse CRC as little endian', (t) => {
  t.is(parseHeader('crc', '1234'), 0x3412)
  t.is(parseHeader('crc', '3412'), 0x1234)
})

test('ArchiveHeaderParser.parse should parse type as 1 byte', (t) => {
  t.is(parseHeader('crc', 'FF'), 0xFF)
  t.is(parseHeader('crc', 'AB'), 0xAB)
  t.is(parseHeader('crc', '00'), 0x00)
  t.is(parseHeader('crc', '0F'), 0x0F)
})

test('ArchiveHeaderParser.parse should parse flags as 2 bytes', t => {
  t.is(hammingWeight(parseHeader('flags', 'ffffff0000')), 0)
  t.is(hammingWeight(parseHeader('flags', 'fffffff000')), 4)
  t.is(hammingWeight(parseHeader('flags', 'ffffffff00')), 8)
  t.is(hammingWeight(parseHeader('flags', 'fffffffff0')), 12)
  t.is(hammingWeight(parseHeader('flags', 'ffffffffff')), 16)
})

test('ArchiveHeaderParser.parse should parse flags as little endian', t => {
  t.is(parseHeader('flags', 'ffffff1234'), 0x3412)
  t.is(parseHeader('flags', 'ffffff3412'), 0x1234)
})

test('ArchiveHeaderParser.parse should parse size as 1 byte', t => {
  t.is(parseHeader('size', 'ffffffff00ff'), 0xFF)
  t.is(parseHeader('size', 'ffffffff000f'), 0x0F)
  t.is(parseHeader('size', 'ffffffff0000'), 0x0D)
  t.is(parseHeader('size', 'ffffffff00AB'), 0xAB)
})

test('ArchiveHeaderParser.parse should parse reserved1 as 2 bytes', t => {
  t.is(hammingWeight(parseHeader('reserved1', 'ffffffffffff00ffff')), 16)
  t.is(hammingWeight(parseHeader('reserved1', 'ffffffffffff00fff0')), 12)
  t.is(hammingWeight(parseHeader('reserved1', 'ffffffffffff00ff00')), 8)
  t.is(hammingWeight(parseHeader('reserved1', 'ffffffffffff00f000')), 4)
  t.is(hammingWeight(parseHeader('reserved1', 'ffffffffffffff0000')), 0)
})

test('ArchiveHeaderParser.parse should parse reserved1 as little endian', t => {
  t.is(parseHeader('reserved1', 'ffffffffffffff1234'), 0x3412)
  t.is(parseHeader('reserved1', 'ffffffffffffff3412'), 0x1234)
})

test('ArchiveHeaderParser.parse should parse reserved2 as 4 bytes', t => {
  t.is(hammingWeight(parseHeader('reserved2', 'ffffffffffffffff00ffffffffff')), 32)
  t.is(hammingWeight(parseHeader('reserved2', 'ffffffffffffffff00ffffff00ff')), 24)
  t.is(hammingWeight(parseHeader('reserved2', 'ffffffffffffffff00ffff0000ff')), 16)
  t.is(hammingWeight(parseHeader('reserved2', 'ffffffffffffffff00ff000000ff')), 8)
  t.is(hammingWeight(parseHeader('reserved2', 'ffffffffffffffff0000000000ff')), 0)
})

test('ArchiveHeaderParser.parse should parse reserved2 as little endian', t => {
  t.is(parseHeader('reserved2', 'ffffffffffffffffff12345678'), 0x78563412)
  t.is(parseHeader('reserved2', 'ffffffffffffffffff78563412'), 0x12345678)
})

test('ArchiveHeaderParser.parse should parse hasVolumeAttributes flag', t => {
  t.truthy(parseHeader('hasVolumeAttributes', btoh(0b100000000000000000000000001)))
  t.falsy(parseHeader('hasVolumeAttributes', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse hasComment flag', t => {
  t.truthy(parseHeader('hasComment', btoh(0b100000000000000000000000010)))
  t.falsy(parseHeader('hasComment', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse isLocked flag', t => {
  t.truthy(parseHeader('isLocked', btoh(0b100000000000000000000000100)))
  t.falsy(parseHeader('isLocked', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse hasSolidAttributes flag', t => {
  t.truthy(parseHeader('hasSolidAttributes', btoh(0b100000000000000000000001000)))
  t.falsy(parseHeader('hasSolidAttributes', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse isNewNameScheme flag', t => {
  t.truthy(parseHeader('isNewNameScheme', btoh(0b100000000000000000000010000)))
  t.falsy(parseHeader('isNewNameScheme', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse hasAuthInfo flag', t => {
  t.truthy(parseHeader('hasAuthInfo', btoh(0b100000000000000000000100000)))
  t.falsy(parseHeader('hasAuthInfo', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse hasRecovery flag', t => {
  t.truthy(parseHeader('hasRecovery', btoh(0b100000000000000000001000000)))
  t.falsy(parseHeader('hasRecovery', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse isBlockEncoded flag', t => {
  t.truthy(parseHeader('isBlockEncoded', btoh(0b100000000000000000010000000)))
  t.falsy(parseHeader('isBlockEncoded', btoh(0b100000000000000000000000000)))
})

test('ArchiveHeaderParser.parse should parse isFirstVolume flag', t => {
  t.truthy(parseHeader('isFirstVolume', btoh(0b100000000000000000000000000000001)))
  t.falsy(parseHeader('isFirstVolume', btoh(0b100000000000000000000000000000000)))
})
