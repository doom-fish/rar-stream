/// <reference types="node" resolution-mode="require"/>
import { IParser, IParsers } from "../../interfaces.js";
export declare const bufferFromString: (str: string, size?: number, variant?: BufferEncoding) => Buffer;
export declare const bind: <T extends IParsers>(Parser: IParser<T>) => {
    parseHeader(field: any, binaryStr: string): any;
};
export declare const newPadding: (count: number) => string;
export declare const hammingWeight: (num: number) => number;
export declare const btoh: (binary: number) => string;
//# sourceMappingURL=utils.d.ts.map