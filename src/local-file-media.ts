import { basename } from "path";
import { statSync, createReadStream } from "fs";
import { IFileMedia } from "./interfaces";

export class LocalFileMedia implements IFileMedia {
  name: string;
  length: number;
  constructor(private path: string) {
    this.name = basename(path);
    this.length = statSync(path).size;
  }
  createReadStream(interval) {
    return createReadStream(this.path, interval);
  }
}
