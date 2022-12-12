import { expect, test } from "vitest";
import { FileHeaderParser } from "./file-header-parser";
import { bind, newPadding, hammingWeight, btoh } from "./__mocks__/utils";
const { parseHeader } = bind(FileHeaderParser);

test("FileHeaderParser.parse should parse crc as 2 bytes", () => {
  expect(hammingWeight(parseHeader("crc", "ffff"))).toBe(16);
  expect(hammingWeight(parseHeader("crc", "fff0"))).toBe(12);
  expect(hammingWeight(parseHeader("crc", "ff00"))).toBe(8);
  expect(hammingWeight(parseHeader("crc", "f000"))).toBe(4);
  expect(hammingWeight(parseHeader("crc", "0000"))).toBe(0);
});

test("FileHeaderParser.parse should parse crc as little endian", () => {
  expect(parseHeader("crc", "1234")).toBe(0x3412);
  expect(parseHeader("crc", "3412")).toBe(0x1234);
});

test("FileHeaderParser.parse should parse type as 1 byte", () => {
  const padding = newPadding(2);

  expect(parseHeader("type", padding + "ff")).toBe(0xff);
  expect(parseHeader("type", padding + "0f")).toBe(0x0f);
  expect(parseHeader("type", padding + "ab")).toBe(0xab);
  expect(parseHeader("type", padding + "ba")).toBe(0xba);
});

test("FileHeaderParser.parse should parse flags as 2 bytes", () => {
  const padding = newPadding(3);
  expect(hammingWeight(parseHeader("flags", padding + "ffff"))).toBe(16);
  expect(hammingWeight(parseHeader("flags", padding + "fff0"))).toBe(12);
  expect(hammingWeight(parseHeader("flags", padding + "ff00"))).toBe(8);
  expect(hammingWeight(parseHeader("flags", padding + "f000"))).toBe(4);
  expect(hammingWeight(parseHeader("flags", padding + "0000"))).toBe(0);
});

test("FileHeaderParser.parse should parse flags as little endian", () => {
  const padding = newPadding(3);
  expect(parseHeader("flags", padding + "3412")).toBe(0x1234);
  expect(parseHeader("flags", padding + "1234")).toBe(0x3412);
});

test("FileHeaderParser.parse should parse headSize as 2 bytes", () => {
  const padding = newPadding(5);
  expect(hammingWeight(parseHeader("headSize", padding + "ffff"))).toBe(16);
  expect(hammingWeight(parseHeader("headSize", padding + "fff0"))).toBe(12);
  expect(hammingWeight(parseHeader("headSize", padding + "ff00"))).toBe(8);
  expect(hammingWeight(parseHeader("headSize", padding + "f000"))).toBe(4);
  expect(hammingWeight(parseHeader("headSize", padding + "0000"))).toBe(0);
});

test("FileHeaderParser.parse should parse headSize as little endian", () => {
  const padding = newPadding(5);
  expect(parseHeader("headSize", padding + "3412")).toBe(0x1234);
  expect(parseHeader("headSize", padding + "1234")).toBe(0x3412);
});

test("FileHeaderParser.parse should parse size as 4 bytes", () => {
  const padding = newPadding(7);
  expect(hammingWeight(parseHeader("size", padding + "ffffffff"))).toBe(32);
  expect(hammingWeight(parseHeader("size", padding + "ffffff00"))).toBe(24);
  expect(hammingWeight(parseHeader("size", padding + "ffff0000"))).toBe(16);
  expect(hammingWeight(parseHeader("size", padding + "ff000000"))).toBe(8);
  expect(hammingWeight(parseHeader("size", padding + "00000000"))).toBe(0);
});

test("FileHeaderParser.parse should parse size as little endian", () => {
  const padding = newPadding(7);
  expect(parseHeader("size", padding + "78563412")).toBe(0x12345678);
  expect(parseHeader("size", padding + "12345678")).toBe(0x78563412);
});

test("FileHeaderParser.parse should parse unpackedSize as 4 bytes", () => {
  const padding = newPadding(11);
  expect(hammingWeight(parseHeader("unpackedSize", padding + "ffffffff"))).toBe(
    32
  );
  expect(hammingWeight(parseHeader("unpackedSize", padding + "ffffff00"))).toBe(
    24
  );
  expect(hammingWeight(parseHeader("unpackedSize", padding + "ffff0000"))).toBe(
    16
  );
  expect(hammingWeight(parseHeader("unpackedSize", padding + "ff000000"))).toBe(
    8
  );
  expect(hammingWeight(parseHeader("unpackedSize", padding + "00000000"))).toBe(
    0
  );
});

test("FileHeaderParser.parse should parse unpackedSize as little endian", () => {
  const padding = newPadding(11);
  expect(parseHeader("unpackedSize", padding + "78563412")).toBe(0x12345678);
  expect(parseHeader("unpackedSize", padding + "12345678")).toBe(0x78563412);
});

test("FileHeaderParser.parse should parse host as 1 byte", () => {
  const padding = newPadding(15);
  expect(parseHeader("host", padding + "ff")).toBe(0xff);
});

test("FileHeaderParser.parse should parse fileCrc as 4 bytes", () => {
  const padding = newPadding(16);
  expect(hammingWeight(parseHeader("fileCrc", padding + "ffffffff"))).toBe(32);
  expect(hammingWeight(parseHeader("fileCrc", padding + "ffffff00"))).toBe(24);
  expect(hammingWeight(parseHeader("fileCrc", padding + "ffff0000"))).toBe(16);
  expect(hammingWeight(parseHeader("fileCrc", padding + "ff000000"))).toBe(8);
  expect(hammingWeight(parseHeader("fileCrc", padding + "00000000"))).toBe(0);
});

test("FileHeaderParser.parse should parse fileCrc as little endian", () => {
  const padding = newPadding(16);
  expect(parseHeader("fileCrc", padding + "78563412")).toBe(0x12345678);
  expect(parseHeader("fileCrc", padding + "12345678")).toBe(0x78563412);
});

test("FileHeaderParser.parse should parse timestamp as 4 bytes", () => {
  const padding = newPadding(20);
  expect(hammingWeight(parseHeader("timestamp", padding + "ffffffff"))).toBe(
    32
  );
  expect(hammingWeight(parseHeader("timestamp", padding + "ffffff00"))).toBe(
    24
  );
  expect(hammingWeight(parseHeader("timestamp", padding + "ffff0000"))).toBe(
    16
  );
  expect(hammingWeight(parseHeader("timestamp", padding + "ff000000"))).toBe(8);
  expect(hammingWeight(parseHeader("timestamp", padding + "00000000"))).toBe(0);
});

test("FileHeaderParser.parse should parse timestamp as little endian", () => {
  const padding = newPadding(20);
  expect(parseHeader("timestamp", padding + "78563412")).toBe(0x12345678);
  expect(parseHeader("timestamp", padding + "12345678")).toBe(0x78563412);
});

test("FileHeaderParser.parse should parse version as 1 bytes", () => {
  const padding = newPadding(24);
  expect(parseHeader("version", padding + "ff")).toBe(0xff);
  expect(parseHeader("version", padding + "ab")).toBe(0xab);
  expect(parseHeader("version", padding + "dd")).toBe(0xdd);
  expect(parseHeader("version", padding + "00")).toBe(0x0);
});

test("FileHeaderParser.parse should parse method as 1 bytes", () => {
  const padding = newPadding(25);
  expect(parseHeader("method", padding + "ff")).toBe(0xff);
  expect(parseHeader("method", padding + "ab")).toBe(0xab);
  expect(parseHeader("method", padding + "dd")).toBe(0xdd);
  expect(parseHeader("method", padding + "00")).toBe(0x0);
});

test("FileHeaderParser.parse should parse nameSize as 2 bytes", () => {
  const padding = newPadding(26);
  expect(hammingWeight(parseHeader("nameSize", padding + "ffff"))).toBe(16);
  expect(hammingWeight(parseHeader("nameSize", padding + "fff0"))).toBe(12);
  expect(hammingWeight(parseHeader("nameSize", padding + "ff00"))).toBe(8);
  expect(hammingWeight(parseHeader("nameSize", padding + "f000"))).toBe(4);
  expect(hammingWeight(parseHeader("nameSize", padding + "0000"))).toBe(0);
});

test("FileHeaderParser.parse should parse nameSize as little endian", () => {
  const padding = newPadding(26);
  expect(parseHeader("nameSize", padding + "1234")).toBe(0x3412);
  expect(parseHeader("nameSize", padding + "3412")).toBe(0x1234);
});

test("FileHeaderParser.parse should parse attributes as 2 bytes", () => {
  const padding = newPadding(28);
  expect(hammingWeight(parseHeader("attributes", padding + "ffffffff"))).toBe(
    32
  );
  expect(hammingWeight(parseHeader("attributes", padding + "ffffff00"))).toBe(
    24
  );
  expect(hammingWeight(parseHeader("attributes", padding + "ffff0000"))).toBe(
    16
  );
  expect(hammingWeight(parseHeader("attributes", padding + "ff000000"))).toBe(
    8
  );
  expect(hammingWeight(parseHeader("attributes", padding + "00000000"))).toBe(
    0
  );
});

test("FileHeaderParser.parse should parse attributes as little endian", () => {
  const padding = newPadding(28);
  expect(parseHeader("attributes", padding + "78563412")).toBe(0x12345678);
  expect(parseHeader("attributes", padding + "12345678")).toBe(0x78563412);
});

test("FileHeaderParser.parse should parse continuesFromPrevious flag", () => {
  const bitField = 0b10000000000000000000000000000001;
  expect(parseHeader("continuesFromPrevious", btoh(bitField))).toBeTruthy();
  expect(parseHeader("continuesFromPrevious", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse continuesInNext flag", () => {
  const bitField = 0b10000000000000000000000000000010;
  expect(parseHeader("continuesInNext", btoh(bitField))).toBeTruthy();
  expect(parseHeader("continuesInNext", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse isEncrypted flag", () => {
  const bitField = 0b10000000000000000000000000000100;
  expect(parseHeader("isEncrypted", btoh(bitField))).toBeTruthy();
  expect(parseHeader("isEncrypted", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse hasComment flag", () => {
  const bitField = 0b10000000000000000000000000001000;
  expect(parseHeader("hasComment", btoh(bitField))).toBeTruthy();
  expect(parseHeader("hasComment", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse hasInfoFromPrevious flag", () => {
  const bitField = 0b10000000000000000000000000010000;
  expect(parseHeader("hasInfoFromPrevious", btoh(bitField))).toBeTruthy();
  expect(parseHeader("hasInfoFromPrevious", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse hasHighSize flag", () => {
  const bitField = 0b1000000000000000000000000000000000000001;
  expect(parseHeader("hasHighSize", btoh(bitField))).toBeTruthy();
  expect(parseHeader("hasHighSize", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse hasSpecialName flag", () => {
  const bitField = 0b1000000000000000000000000000000000000010;
  expect(parseHeader("hasSpecialName", btoh(bitField))).toBeTruthy();
  expect(parseHeader("hasSpecialName", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse hasSalt flag", () => {
  const bitField = 0b1000000000000000000000000000000000000100;
  expect(parseHeader("hasSalt", btoh(bitField))).toBeTruthy();
  expect(parseHeader("hasSalt", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse isOldVersion flag", () => {
  const bitField = 0b1000000000000000000000000000000000001000;
  expect(parseHeader("isOldVersion", btoh(bitField))).toBeTruthy();
  expect(parseHeader("isOldVersion", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should parse hasExtendedTime flag", () => {
  const bitField = 0b1000000000000000000000000000000000010000;
  expect(parseHeader("hasExtendedTime", btoh(bitField))).toBeTruthy();
  expect(parseHeader("hasExtendedTime", "00")).toBeFalsy();
});

test("FileHeaderParser.parse should handle high file size", () => {
  const data =
    "D97774111111115C1000005C10000003C5A6D2158A5" +
    "95B4714300A00A4810000040000000400000061636B6" +
    "E6F772E74787400C0";

  expect(parseHeader("hasHighSize", data)).toBeTruthy();
  expect(parseHeader("size", data)).toBe(0x40000105c);
  expect(parseHeader("unpackedSize", data)).toBe(0x40000105c);
});

test("FileHeaderParser.parse should parse name properly", () => {
  const data =
    "D97774111111115C1000005C10000003C5A6D2158A5" +
    "95B4714300A00A4810000040000000400000061636B6" +
    "E6F772E74787400C0";
  expect(parseHeader("name", data), "ackno).toBe(txt");
});
