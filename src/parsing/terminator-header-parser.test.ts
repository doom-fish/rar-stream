import { expect, test } from "vitest";
import { TerminatorHeaderParser } from "./terminator-header-parser";
import { bind, hammingWeight } from "./__mocks__/utils";
const { parseHeader } = bind(TerminatorHeaderParser);

test("TerminatorHeaderParser.parse should parse 2 first bytes as crc", () => {
  expect(hammingWeight(parseHeader("crc", "ffffAB"))).toBe(16);
  expect(hammingWeight(parseHeader("crc", "ffff00"))).toBe(16);
  expect(hammingWeight(parseHeader("crc", "ffffff"))).toBe(16);
  expect(hammingWeight(parseHeader("crc", "ffffAB"))).toBe(16);
  expect(hammingWeight(parseHeader("crc", "ff0000"))).toBe(8);
  expect(hammingWeight(parseHeader("crc", "f00000"))).toBe(4);
  expect(hammingWeight(parseHeader("crc", "000000"))).toBe(0);
  expect(parseHeader("crc", "ffffAB")).toBe(0xffff);
});

test("TerminatorHeaderParser.parse should parse in little endian", () => {
  expect(parseHeader("crc", "3412")).toBe(0x1234);
  expect(parseHeader("crc", "1234")).toBe(0x3412);
});

test("TerminatorHeaderParser.parse should parse type as 1 byte", () => {
  expect(hammingWeight(parseHeader("type", "FFFFFF"))).toBe(8);
  expect(hammingWeight(parseHeader("type", "FFFF00"))).toBe(0);
  expect(parseHeader("type", "FFFFFF")).toBe(0xff);
  expect(parseHeader("type", "FFFFFA")).toBe(0xfa);
  expect(parseHeader("type", "FFFF0A")).toBe(0x0a);
});

test("TerminatorHeaderParser.parse should parse flags as 2 bytes", () => {
  expect(hammingWeight(parseHeader("flags", "ffffffffff"))).toBe(16);
  expect(hammingWeight(parseHeader("flags", "ffffff0000"))).toBe(0);
  expect(hammingWeight(parseHeader("flags", "ffffff000F"))).toBe(4);
  expect(hammingWeight(parseHeader("flags", "ffffff00F0"))).toBe(4);
  expect(hammingWeight(parseHeader("flags", "ffffff0F00"))).toBe(4);
  expect(hammingWeight(parseHeader("flags", "fffffff000"))).toBe(4);

  expect(parseHeader("flags", "ffffff0000")).toBe(0);
  expect(parseHeader("flags", "ffffffffff")).toBe(0xffff);
});

test("TerminatorHeaderParser.parse should parse flags as little endian", () => {
  expect(parseHeader("flags", "ffffff00FF")).toBe(0xff00);
  expect(parseHeader("flags", "ffffffff00")).toBe(0x00ff);
  expect(parseHeader("flags", "ffffff1234")).toBe(0x3412);
  expect(parseHeader("flags", "ffffff3412")).toBe(0x1234);
});

test("TerminatorHeaderParser.parse should parse size as 2 bytes", () => {
  expect(hammingWeight(parseHeader("size", "ffffffffffffff"))).toBe(16);
  expect(hammingWeight(parseHeader("size", "ffffffffff0000"))).toBe(0);
  expect(hammingWeight(parseHeader("size", "ffffffffff000F"))).toBe(4);
  expect(hammingWeight(parseHeader("size", "ffffffffff00F0"))).toBe(4);
  expect(hammingWeight(parseHeader("size", "ffffffffff0F00"))).toBe(4);
  expect(hammingWeight(parseHeader("size", "fffffffffff000"))).toBe(4);

  expect(parseHeader("size", "ffffffffff0000")).toBe(0);
  expect(parseHeader("size", "ffffffffffffff")).toBe(0xffff);
});

test("TerminatorHeaderParser.parse should parse size as little endian", () => {
  expect(parseHeader("size", "ffffffffff00FF")).toBe(0xff00);
  expect(parseHeader("size", "ffffffffffff00")).toBe(0x00ff);
  expect(parseHeader("size", "ffffffffff1234")).toBe(0x3412);
  expect(parseHeader("size", "ffffffffff3412")).toBe(0x1234);
});
