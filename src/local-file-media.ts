import { basename } from "path";
import { statSync, createReadStream } from "fs";
import { IFileMedia, IReadInterval } from "./interfaces.js";

export class LocalFileMedia implements IFileMedia {
  name: string;
  length: number;
  constructor(private path: string) {
    this.name = basename(path);
    this.length = statSync(path).size;
  }
  createReadStream(interval: IReadInterval) {
    return Promise.resolve(
      createReadStream(this.path, interval)
    );
  }
}
