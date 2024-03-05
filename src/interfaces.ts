import { ArchiveHeaderParser } from "./parsing/archive-header-parser.js";
import { FileHeaderParser } from "./parsing/file-header-parser.js";
import { MarkerHeaderParser } from "./parsing/marker-header-parser.js";
import { TerminatorHeaderParser } from "./parsing/terminator-header-parser.js";
export interface IFileMedia {
  length: number;
  name: string;
  createReadStream(opts?: IReadInterval): Promise<NodeJS.ReadableStream> | NodeJS.ReadableStream;
}
export interface IReadInterval {
  start: number;
  end: number;
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
