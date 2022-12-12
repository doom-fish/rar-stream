import { expect, test } from "vitest";
import { MarkerHeaderParser } from "./marker-header-parser";
import { bind, newPadding, hammingWeight } from "./__mocks__/utils";
const { parseHeader } = bind(MarkerHeaderParser);

test("MarkerHeaderParser.parse should parse crc as 2 bytes", () => {
  expect(hammingWeight(parseHeader("crc", "ffff"))).toBe(16);
  expect(hammingWeight(parseHeader("crc", "fff0"))).toBe(12);
  expect(hammingWeight(parseHeader("crc", "ff00"))).toBe(8);
  expect(hammingWeight(parseHeader("crc", "f000"))).toBe(4);
  expect(hammingWeight(parseHeader("crc", "0000"))).toBe(0);
});

test("MarkerHeaderParser.parse should parse crc as little endian", () => {
  expect(parseHeader("crc", "1234")).toBe(0x3412);
  expect(parseHeader("crc", "3412")).toBe(0x1234);
});

test("MarkerHeaderParser.parse should parse type as 1 byte", () => {
  const padding = newPadding(2);
  expect(parseHeader("type", padding + "72")).toBe(0x72);
  expect(parseHeader("type", padding + "ff")).toBe(0xff);
  expect(parseHeader("type", padding + "01")).toBe(0x01);
});

test("MarkerHeaderParser.parse should parse flags as 2 bytes", () => {
  const padding = newPadding(3);
  expect(hammingWeight(parseHeader("flags", padding + "ffff"))).toBe(16);
  expect(hammingWeight(parseHeader("flags", padding + "fff0"))).toBe(12);
  expect(hammingWeight(parseHeader("flags", padding + "ff00"))).toBe(8);
  expect(hammingWeight(parseHeader("flags", padding + "f000"))).toBe(4);
  expect(hammingWeight(parseHeader("flags", padding + "0000"))).toBe(0);
});

test("MarkerHeaderParser.parse should parse flags as little endian", () => {
  const padding = newPadding(3);
  expect(parseHeader("flags", padding + "1234")).toBe(0x3412);
  expect(parseHeader("flags", padding + "3412")).toBe(0x1234);
});

test("MarkerHeaderParser.parse should parse size as 1 byte", () => {
  const padding = newPadding(5);
  expect(parseHeader("size", padding + "ff")).toBe(0xff);
  expect(parseHeader("size", padding + "ab")).toBe(0xab);
  expect(parseHeader("size", padding + "f0")).toBe(0xf0);
  expect(parseHeader("size", padding + "f1")).toBe(0xf1);
});

test("MarkerHeaderParser.parse should parse add_size flag", () => {
  expect(parseHeader("size", "526172219A070001000000")).toBe(0x08);
  expect(parseHeader("size", "526172219A070009000000")).toBe(0x10);
  expect(parseHeader("size", "526172219A07000A000000")).toBe(0x11);
  expect(parseHeader("size", "526172219A0700F8FFFFFF")).toBe(0xffffffff);
});
