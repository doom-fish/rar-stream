import { ArchiveHeaderParser } from "./parsing/archive-header-parser.js";
import { FileHeaderParser } from "./parsing/file-header-parser.js";
import { MarkerHeaderParser } from "./parsing/marker-header-parser.js";
import { TerminatorHeaderParser } from "./parsing/terminator-header-parser.js";
export interface IFileMedia {
  length: number;
  name: string;
  createReadStream(opts?: IReadInterval): NodeJS.ReadableStream;
}
export interface IReadInterval {
  start: number;
  end: number;
}
export interface FindOpts {
  filter(
    filename: string,
    idx: number
  ): boolean;
  maxFiles: number;
  fileIdx: number;
}

export type IParsers =
  | ArchiveHeaderParser
  | FileHeaderParser
  | MarkerHeaderParser
  | TerminatorHeaderParser;
export type IParser<T extends IParsers> = {
  HEADER_SIZE: number;
  new (buffer: Buffer): T;
};
