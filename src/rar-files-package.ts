import { EventEmitter } from "events";
import { makeRarFileBundle, RarFileBundle } from "./rar-file-bundle.js";
import { RarFileChunk } from "./rar-file-chunk.js";
import { InnerFile } from "./inner-file.js";

import { MarkerHeaderParser } from "./parsing/marker-header-parser.js";
import { ArchiveHeaderParser } from "./parsing/archive-header-parser.js";
import { FileHeaderParser, IFileHeader } from "./parsing/file-header-parser.js";
import { TerminatorHeaderParser } from "./parsing/terminator-header-parser.js";

import { streamToBuffer } from "./stream-utils.js";
import { IFileMedia, IParser, IParsers } from "./interfaces.js";
import { groupBy, mapValues } from "./utils.js";

const parseHeader = async <T extends IParsers>(
  Parser: IParser<T>,
  fileMedia: IFileMedia,
  offset = 0
) => {
  const stream = fileMedia.createReadStream({
    start: offset,
    end: offset + Parser.HEADER_SIZE,
  });
  const headerBuffer = await streamToBuffer(stream);
  const parser = new Parser(headerBuffer);
  return parser.parse() as ReturnType<T["parse"]>;
};
interface ParsedFileChunkMapping {
  name: string;
  chunk: RarFileChunk;
}
interface FileChunkMapping extends ParsedFileChunkMapping {
  fileHead: IFileHeader;
}

export class RarFilesPackage extends EventEmitter {
  rarFileBundle: RarFileBundle;
  constructor(fileMedias: IFileMedia[]) {
    super();
    this.rarFileBundle = makeRarFileBundle(fileMedias);
  }
  async parseFile(rarFile: IFileMedia) {
    const fileChunks: FileChunkMapping[] = [];
    let fileOffset = 0;
    const markerHead = await parseHeader(MarkerHeaderParser, rarFile);
    fileOffset += markerHead.size;

    const archiveHeader = await parseHeader(
      ArchiveHeaderParser,
      rarFile,
      fileOffset
    );
    fileOffset += archiveHeader.size;

    while (fileOffset < rarFile.length - TerminatorHeaderParser.HEADER_SIZE) {
      const fileHead = await parseHeader(FileHeaderParser, rarFile, fileOffset);
      if (fileHead.type !== 116) {
        break;
      }
      if (fileHead.method !== 0x30) {
        throw new Error("Decompression is not implemented");
      }
      fileOffset += fileHead.headSize;

      fileChunks.push({
        name: fileHead.name,
        fileHead,
        chunk: new RarFileChunk(
          rarFile,
          fileOffset,
          fileOffset + fileHead.size - 1
        ),
      });
      fileOffset += fileHead.size;
    }
    this.emit("file-parsed", rarFile);
    return fileChunks;
  }
  async parse(): Promise<InnerFile[]> {
    this.emit("parsing-start", this.rarFileBundle);
    const parsedFileChunks: ParsedFileChunkMapping[][] = [];
    const { files } = this.rarFileBundle;
    for (let i = 0; i < files.length; ++i) {
      const file = files[i]!;

      const chunks = await this.parseFile(file);
      const { fileHead, chunk } = chunks[chunks.length - 1]!;
      const chunkSize = Math.abs(chunk.endOffset - chunk.startOffset);
      let innerFileSize = fileHead.unpackedSize;
      parsedFileChunks.push(chunks);

      if (fileHead.continuesInNext) {
        while (Math.abs(innerFileSize - chunkSize) >= chunkSize) {
          const nextFile = files[++i]!;

          parsedFileChunks.push([
            {
              name: fileHead.name,
              chunk: new RarFileChunk(
                nextFile,
                chunk.startOffset,
                chunk.endOffset
              ),
            },
          ]);
          this.emit("file-parsed", nextFile);
          innerFileSize -= chunkSize;
        }
      }
    }

    const fileChunks = parsedFileChunks.flat();

    const grouped = mapValues(
      groupBy(fileChunks, (f) => f.name),
      (value) => value.map((v) => v.chunk)
    );

    const innerFiles = Object.entries(grouped).map(
      ([name, chunks]) => new InnerFile(name, chunks)
    );

    this.emit("parsing-complete", innerFiles);
    return innerFiles;
  }
}
