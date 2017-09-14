const { EventEmitter } = require('events');
const makeRarFileBundle = require('./rar-file-bundle');
const RarFileChunk = require('./rar-file-chunk');

const InnerFile = require('./inner-file');

const MarkerHeaderParser = require('./parsing/marker-header-parser');
const ArchiveHeaderParser = require('./parsing/archive-header-parser');
const FileHeaderParser = require('./parsing/file-header-parser');
const TerminalHeaderParser = require('./parsing/terminator-header-parser');

const { streamToBuffer } = require('./stream-utils');

const flatten = list =>
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

module.exports = class RarFilesPackage extends EventEmitter {
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

    while (fileOffset < rarFile.length - TerminalHeaderParser.HEADER_SIZE) {
      const fileHead = await parseHeader(FileHeaderParser, rarFile, fileOffset);
      if (fileHead.type !== 116) {
        break;
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
    this.emit('file-parsed', rarFile);
    return fileChunks;
  }
  async parse() {
    this.emit('parsing-start', this.rarFileBundle);
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
          this.emit('file-parsed', nextFile);
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
      name => new InnerFile(name, grouped[name])
    );

    this.emit('parsing-complete', innerFiles);
    return innerFiles;
  }
};
