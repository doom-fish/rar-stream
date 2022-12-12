import { EventEmitter } from "events";
import { makeRarFileBundle } from "./rar-file-bundle.js";
import { RarFileChunk } from "./rar-file-chunk.js";
import { InnerFile } from "./inner-file.js";

import { MarkerHeaderParser } from "./parsing/marker-header-parser.js";
import { ArchiveHeaderParser } from "./parsing/archive-header-parser.js";
import { FileHeaderParser } from "./parsing/file-header-parser.js";
import { TerminatorHeaderParser } from "./parsing/terminator-header-parser.js";

import { streamToBuffer } from "./stream-utils.js";

const flatten = (list) =>
  list.reduce((a, b) => a.concat(Array.isArray(b) ? flatten(b) : b), []);

const parseHeader = async (Parser, fileMedia, offset = 0) => {
  const stream = fileMedia.createReadStream({
    start: offset,
    end: offset + Parser.HEADER_SIZE,
  });
  const headerBuffer = await streamToBuffer(stream);
  const parser = new Parser(headerBuffer);
  return parser.parse();
};

export class RarFilesPackage extends EventEmitter {
  constructor(fileMedias) {
    super();
    this.rarFileBundle = makeRarFileBundle(fileMedias);
  }
  async parseFile(rarFile) {
    const fileChunks = [];
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
    const parsedFileChunks = [];
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

    const fileChunks = flatten(parsedFileChunks);

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
