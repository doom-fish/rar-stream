import { IFileMedia } from "./interfaces.js";
declare class NumericRarFileBundle {
    private fileMedias;
    constructor(fileMedias?: IFileMedia[]);
    filter(): void;
    sort(): void;
    get length(): number;
    get fileNames(): string[];
    get files(): IFileMedia[];
}
declare class PartXXRarBundle {
    private fileMedias;
    constructor(fileMedias?: IFileMedia[]);
    filter(): void;
    sort(): void;
    get length(): number;
    get fileNames(): string[];
    get files(): IFileMedia[];
}
export type RarFileBundle = PartXXRarBundle | NumericRarFileBundle;
export declare const makeRarFileBundle: (fileMedias?: IFileMedia[]) => RarFileBundle;
export {};
//# sourceMappingURL=rar-file-bundle.d.ts.map