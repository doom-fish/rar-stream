import { IFileMedia } from "./interfaces";

export class RarFileChunk {
  constructor(
    private fileMedia: IFileMedia,
    public startOffset: number,
    public endOffset: number
  ) {}
  padEnd(endPadding: number) {
    return new RarFileChunk(
      this.fileMedia,
      this.startOffset,
      this.endOffset - endPadding
    );
  }
  padStart(startPadding) {
    return new RarFileChunk(
      this.fileMedia,
      this.startOffset + startPadding,
      this.endOffset
    );
  }
  get length() {
    return Math.max(0, this.endOffset - this.startOffset);
  }
  getStream() {
    return this.fileMedia.createReadStream({
      start: this.startOffset,
      end: this.endOffset,
    });
  }
}
