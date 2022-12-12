/// <reference types="node" resolution-mode="require"/>
import { MockFileStream } from "./mock-file-stream.js";
import { IFileMedia, IReadInterval } from "../../interfaces.js";
export declare class MockFileMedia implements IFileMedia {
    buffer: Buffer;
    constructor(stringData: string, name?: string);
    length: number;
    name: string;
    createReadStream(options: IReadInterval): MockFileStream;
}
//# sourceMappingURL=mock-file-media.d.ts.map