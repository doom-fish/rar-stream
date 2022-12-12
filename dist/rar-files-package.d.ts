/// <reference types="node" resolution-mode="require"/>
import { EventEmitter } from "events";
import { RarFileBundle } from "./rar-file-bundle.js";
import { RarFileChunk } from "./rar-file-chunk.js";
import { InnerFile } from "./inner-file.js";
import { IFileHeader } from "./parsing/file-header-parser.js";
import { IFileMedia } from "./interfaces.js";
interface ParsedFileChunkMapping {
    name: string;
    chunk: RarFileChunk;
}
interface FileChunkMapping extends ParsedFileChunkMapping {
    fileHead: IFileHeader;
}
export declare class RarFilesPackage extends EventEmitter {
    rarFileBundle: RarFileBundle;
    constructor(fileMedias: IFileMedia[]);
    parseFile(rarFile: IFileMedia): Promise<FileChunkMapping[]>;
    parse(): Promise<InnerFile[]>;
}
export {};
//# sourceMappingURL=rar-files-package.d.ts.map