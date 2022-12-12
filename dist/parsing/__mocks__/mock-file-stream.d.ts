/// <reference types="node" resolution-mode="require"/>
/// <reference types="node" resolution-mode="require"/>
import { Readable } from "stream";
import { IReadInterval } from "../../interfaces.js";
export declare class MockFileStream extends Readable {
    private object;
    private options;
    constructor(object: Buffer | null, options: IReadInterval);
    _read(): void;
}
//# sourceMappingURL=mock-file-stream.d.ts.map