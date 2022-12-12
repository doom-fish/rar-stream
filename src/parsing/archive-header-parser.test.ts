import { expect, test } from "vitest";

import { ArchiveHeaderParser } from "./archive-header-parser";
import { bind, hammingWeight, btoh } from "./__mocks__/utils";
const { parseHeader } = bind(ArchiveHeaderParser);

test("ArchiveHeaderParser.parse should parse CRC as 2 bytes", () => {
  expect(hammingWeight(parseHeader("crc", "ffff00ffff"))).toBe(16);
  expect(hammingWeight(parseHeader("crc", "ff0000ffff"))).toBe(8);
  expect(hammingWeight(parseHeader("crc", "f00000ffff"))).toBe(4);
  expect(hammingWeight(parseHeader("crc", "000000ffff"))).toBe(0);
});

test("ArchiveHeaderParser.parse should parse CRC as little endian", () => {
  expect(parseHeader("crc", "1234")).toBe(0x3412);
  expect(parseHeader("crc", "3412")).toBe(0x1234);
});

test("ArchiveHeaderParser.parse should parse type as 1 byte", () => {
  expect(parseHeader("type", "0000FF")).toBe(0xff);
  expect(parseHeader("type", "0000AB")).toBe(0xab);
  expect(parseHeader("type", "000000")).toBe(0x00);
  expect(parseHeader("type", "00000F")).toBe(0x0f);
});

test("ArchiveHeaderParser.parse should parse flags as 2 bytes", () => {
  expect(hammingWeight(parseHeader("flags", "ffffff0000"))).toBe(0);
  expect(hammingWeight(parseHeader("flags", "fffffff000"))).toBe(4);
  expect(hammingWeight(parseHeader("flags", "ffffffff00"))).toBe(8);
  expect(hammingWeight(parseHeader("flags", "fffffffff0"))).toBe(12);
  expect(hammingWeight(parseHeader("flags", "ffffffffff"))).toBe(16);
});

test("ArchiveHeaderParser.parse should parse flags as little endian", () => {
  expect(parseHeader("flags", "ffffff1234")).toBe(0x3412);
  expect(parseHeader("flags", "ffffff3412")).toBe(0x1234);
});

test("ArchiveHeaderParser.parse should parse size as 2 byte", () => {
  expect(parseHeader("size", "ffffffff00ff")).toBe(0xff);
  expect(parseHeader("size", "ffffffff000f")).toBe(0x0f);
  expect(parseHeader("size", "ffffffff0000")).toBe(0x00);
  expect(parseHeader("size", "ffffffff00AB")).toBe(0xab);
});

test("ArchiveHeaderParser.parse should parse reserved1 as 2 bytes", () => {
  expect(hammingWeight(parseHeader("reserved1", "ffffffffffff00ffff"))).toBe(
    16
  );
  expect(hammingWeight(parseHeader("reserved1", "ffffffffffff00fff0"))).toBe(
    12
  );
  expect(hammingWeight(parseHeader("reserved1", "ffffffffffff00ff00"))).toBe(8);
  expect(hammingWeight(parseHeader("reserved1", "ffffffffffff00f000"))).toBe(4);
  expect(hammingWeight(parseHeader("reserved1", "ffffffffffffff0000"))).toBe(0);
});

test("ArchiveHeaderParser.parse should parse reserved1 as little endian", () => {
  expect(parseHeader("reserved1", "ffffffffffffff1234")).toBe(0x3412);
  expect(parseHeader("reserved1", "ffffffffffffff3412")).toBe(0x1234);
});

test("ArchiveHeaderParser.parse should parse reserved2 as 4 bytes", () => {
  expect(
    hammingWeight(parseHeader("reserved2", "ffffffffffffffff00ffffffffff"))
  ).toBe(32);
  expect(
    hammingWeight(parseHeader("reserved2", "ffffffffffffffff00ffffff00ff"))
  ).toBe(24);
  expect(
    hammingWeight(parseHeader("reserved2", "ffffffffffffffff00ffff0000ff"))
  ).toBe(16);
  expect(
    hammingWeight(parseHeader("reserved2", "ffffffffffffffff00ff000000ff"))
  ).toBe(8);
  expect(
    hammingWeight(parseHeader("reserved2", "ffffffffffffffff0000000000ff"))
  ).toBe(0);
});

test("ArchiveHeaderParser.parse should parse reserved2 as little endian", () => {
  expect(parseHeader("reserved2", "ffffffffffffffffff12345678")).toBe(
    0x78563412
  );
  expect(parseHeader("reserved2", "ffffffffffffffffff78563412")).toBe(
    0x12345678
  );
});

test("ArchiveHeaderParser.parse should parse hasVolumeAttributes flag", () => {
  expect(
    parseHeader("hasVolumeAttributes", btoh(0b100000000000000000000000001))
  ).toBeTruthy();
  expect(
    parseHeader("hasVolumeAttributes", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse hasComment flag", () => {
  expect(
    parseHeader("hasComment", btoh(0b100000000000000000000000010))
  ).toBeTruthy();
  expect(
    parseHeader("hasComment", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse isLocked flag", () => {
  expect(
    parseHeader("isLocked", btoh(0b100000000000000000000000100))
  ).toBeTruthy();
  expect(
    parseHeader("isLocked", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse hasSolidAttributes flag", () => {
  expect(
    parseHeader("hasSolidAttributes", btoh(0b100000000000000000000001000))
  ).toBeTruthy();
  expect(
    parseHeader("hasSolidAttributes", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse isNewNameScheme flag", () => {
  expect(
    parseHeader("isNewNameScheme", btoh(0b100000000000000000000010000))
  ).toBeTruthy();
  expect(
    parseHeader("isNewNameScheme", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse hasAuthInfo flag", () => {
  expect(
    parseHeader("hasAuthInfo", btoh(0b100000000000000000000100000))
  ).toBeTruthy();
  expect(
    parseHeader("hasAuthInfo", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse hasRecovery flag", () => {
  expect(
    parseHeader("hasRecovery", btoh(0b100000000000000000001000000))
  ).toBeTruthy();
  expect(
    parseHeader("hasRecovery", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse isBlockEncoded flag", () => {
  expect(
    parseHeader("isBlockEncoded", btoh(0b100000000000000000010000000))
  ).toBeTruthy();
  expect(
    parseHeader("isBlockEncoded", btoh(0b100000000000000000000000000))
  ).toBeFalsy();
});

test("ArchiveHeaderParser.parse should parse isFirstVolume flag", () => {
  expect(
    parseHeader("isFirstVolume", btoh(0b100000000000000000000000000000001))
  ).toBeTruthy();
  expect(
    parseHeader("isFirstVolume", btoh(0b100000000000000000000000000000000))
  ).toBeFalsy();
});
