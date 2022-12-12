/// <reference types="node" resolution-mode="require"/>
import { IFileMedia, IReadInterval } from "./interfaces.js";
export declare class LocalFileMedia implements IFileMedia {
    private path;
    name: string;
    length: number;
    constructor(path: string);
    createReadStream(interval: IReadInterval): import("fs").ReadStream;
}
//# sourceMappingURL=local-file-media.d.ts.map