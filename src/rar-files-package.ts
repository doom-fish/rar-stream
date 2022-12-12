import { EventEmitter } from "events";
import { makeRarFileBundle, RarFileBundle } from "./rar-file-bundle";
import { RarFileChunk } from "./rar-file-chunk";
import { InnerFile } from "./inner-file";

import { MarkerHeaderParser } from "./parsing/marker-header-parser";
import { ArchiveHeaderParser } from "./parsing/archive-header-parser";
import { FileHeaderParser, IFileHeader } from "./parsing/file-header-parser";
import { TerminatorHeaderParser } from "./parsing/terminator-header-parser";

import { streamToBuffer } from "./stream-utils";

const parseHeader = async (Parser, fileMedia, offset = 0) => {
  const stream = fileMedia.createReadStream({
    start: offset,
    end: offset + Parser.HEADER_SIZE,
  });
  const headerBuffer = await streamToBuffer(stream);
  const parser = new Parser(headerBuffer);
  return parser.parse();
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
  constructor(fileMedias) {
    super();
    this.rarFileBundle = makeRarFileBundle(fileMedias);
  }
  async parseFile(rarFile) {
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

    while (
      fileOffset <
      rarFile.length - TerminatorHeaderParser.HEADER_SIZE - 20
    ) {
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
  async parse() {
    this.emit("parsing-start", this.rarFileBundle);
    const parsedFileChunks: ParsedFileChunkMapping[][] = [];
    const { files } = this.rarFileBundle;
    for (let i = 0; i < files.length; ++i) {
      const file = files[i];

      const chunks = await this.parseFile(file);
      const { fileHead, chunk } = chunks[chunks.length - 1];
      const chunkSize = Math.abs(chunk.endOffset - chunk.startOffset);
      let innerFileSize = fileHead.unpackedSize;
      parsedFileChunks.push(chunks);

      if (fileHead.continuesInNext) {
        while (Math.abs(innerFileSize - chunkSize) >= chunkSize) {
          const nextFile = files[++i];

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

    const grouped = fileChunks.reduce((file, { name, chunk }) => {
      if (!file[name]) {
        file[name] = [];
      }

      file[name].push(chunk);
      return file;
    }, {});

    const innerFiles = Object.keys(grouped).map(
      (name) => new InnerFile(name, grouped[name])
    );

    this.emit("parsing-complete", innerFiles);
    return innerFiles;
  }
}
