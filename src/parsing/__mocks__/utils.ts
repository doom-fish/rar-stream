import { ArchiveHeaderParser } from "../archive-header-parser";
import { FileHeaderParser } from "../file-header-parser";
import { MarkerHeaderParser } from "../marker-header-parser";
import { TerminatorHeaderParser } from "../terminator-header-parser";

export const bufferFromString = (
  str: string,
  size?: number,
  variant: BufferEncoding = "hex"
) => {
  if (size) {
    let padding = Math.abs(+size - str.length / 2);
    str += Array(padding).fill("00").join("");
  }

  return Buffer.from(str, variant);
};
export const bind = (Parser: {
  HEADER_SIZE: number;
  new (buffer: Buffer):
    | ArchiveHeaderParser
    | FileHeaderParser
    | MarkerHeaderParser
    | TerminatorHeaderParser;
}) => ({
  parseHeader(field, binaryStr) {
    return new Parser(bufferFromString(binaryStr, Parser.HEADER_SIZE)).parse()[
      field
    ];
  },
});

export const newPadding = (count: number) =>
  Array(count * 2)
    .fill("0")
    .join("");

export const hammingWeight = (num: number) => {
  num = num - ((num >> 1) & 0x55555555);
  num = (num & 0x33333333) + ((num >> 2) & 0x33333333);
  return (((num + (num >> 4)) & 0xf0f0f0f) * 0x1010101) >> 24;
};

export const btoh = (binary: number) => {
  const str = binary.toString(16);
  return str.length % 2 !== 0 ? "0" + str : str;
};
