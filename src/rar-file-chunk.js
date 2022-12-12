export class RarFileChunk {
  constructor(fileMedia, startOffset, endOffset) {
    this.fileMedia = fileMedia;
    this.startOffset = startOffset;
    this.endOffset = endOffset;
  }
  padEnd(endPadding) {
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
