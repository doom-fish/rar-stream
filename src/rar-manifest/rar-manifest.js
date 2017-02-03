// @flow
import RarFileBundle from '../rar-file/rar-file-bundle';
import RarFile from '../rar-file/rar-file';
import RarFileChunk from '../rar-file/rar-file-chunk';
import FileMedia from '../file-media/file-media';
import MarkerHeaderParser from '../parsing/marker-header-parser';
import AchiverHeadParser from '../parsing/archive-header-parser';
import FileHeaderParser from '../parsing/file-header-parser';
import TerminalHeaderParser from '../parsing/terminator-header-parser';

const flatten = list =>
    list.reduce((a, b) => a.concat(Array.isArray(b) ? flatten(b) : b), []);

export default class RarManifest {
    _rarFileBundle: RarFileBundle;
    _rarFiles: RarFile[];
    constructor(rarFileBundle: RarFileBundle) {
        this._rarFileBundle = rarFileBundle;
    }
    async _parseMarkerHead(fileMedia: FileMedia): Promise<*> {
        const interval = {
            start: 0,
            end: MarkerHeaderParser.bytesToRead
        };
        const stream = await fileMedia.createReadStream(interval);
        const parser = new MarkerHeaderParser(stream);
        return parser.parse();
    }
    async _parseArchiveHead(offset: number, fileMedia: FileMedia): Promise<*> {
        const interval = {
            start: offset,
            end: AchiverHeadParser.bytesToRead
        };
        const stream = await fileMedia.createReadStream(interval);
        const parser = new AchiverHeadParser(stream);
        return await parser.parse();
    }
    async _parseFileHead(offset: number, fileMedia: FileMedia): Promise<*> {
        const interval = {
            start: offset,
            end: offset + FileHeaderParser.bytesToRead
        };

        const fileStream = await fileMedia.createReadStream(interval);

        const parser = new FileHeaderParser(fileStream);
        return parser.parse();
    }
    async _parseFile(rarFile: FileMedia): Promise<[]> {
        const fileChunks = [];
        let fileOffset = 0;
        const markerHead = await this._parseMarkerHead(rarFile);
        fileOffset += markerHead.size;

        const archiveHeader = await this._parseArchiveHead(fileOffset, rarFile);
        fileOffset += archiveHeader.size;

        while (fileOffset < rarFile.size - TerminalHeaderParser.bytesToRead) {
            const fileHead = await this._parseFileHead(fileOffset, rarFile);
            if (fileHead.type !== 116) {
                break;
            }
            fileOffset += fileHead.headSize;

            fileChunks.push({
                name: fileHead.name,
                chunk: new RarFileChunk(
                    rarFile,
                    fileOffset,
                    fileOffset + fileHead.size - 1
                )
            });
            fileOffset += fileHead.size;
        }
        return fileChunks;
    }
    async _parse(): Promise<RarFile[]> {
        const fileChunks = flatten(
            await Promise.all(
                this._rarFileBundle.files.map(file => this._parseFile(file))
            )
        );

        const grouped = fileChunks.reduce(
            (file, { name, chunk }) => {
                if (!file[name]) {
                    file[name] = [];
                }
                file[name].push(chunk);
                return file;
            },
            {}
        );

        return Object.keys(grouped).map(
            name => new RarFile(name, grouped[name])
        );
    }
    getFiles(): Promise<RarFile[]> {
        return this._parse();
    }
}
