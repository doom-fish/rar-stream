import { basename } from "path";
import { statSync, createReadStream } from "fs";

export class LocalFileMedia {
  constructor(path) {
    if (typeof path !== "string") {
      throw new Error(
        "Invalid Arguments, path" +
          "need to be passed to the constructor as a string"
      );
    }
    this.path = path;
    this.name = basename(path);
    this.length = statSync(path).size;
  }
  createReadStream(interval) {
    return createReadStream(this.path, interval);
  }
}
