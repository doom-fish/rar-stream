import { Readable } from "stream";
export interface IFileMedia {
  length: number;
  name: string;
  createReadStream(interval: IReadInterval): Readable;
}
export interface IReadInterval {
  start: number;
  end: number;
}
