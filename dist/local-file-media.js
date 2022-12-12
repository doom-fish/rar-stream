import { basename } from "path";
import { statSync, createReadStream } from "fs";
export class LocalFileMedia {
    path;
    name;
    length;
    constructor(path) {
        this.path = path;
        this.name = basename(path);
        this.length = statSync(path).size;
    }
    createReadStream(interval) {
        return createReadStream(this.path, interval);
    }
}
