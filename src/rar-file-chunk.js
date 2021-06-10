module.exports = class RarFileChunk {
  constructor(fileMedia, startOffset, endOffset) {
    this.fileMedia = fileMedia;
    this.startOffset = startOffset;
    this.endOffset = endOffset;
  }
  paddEnd(endPadding) {
    return new RarFileChunk(
      this.fileMedia,
      this.startOffset,
      this.endOffset - endPadding
    );
  }
  paddStart(startPadding) {
    return new RarFileChunk(
      this.fileMedia,
      this.startOffset + startPadding,
      this.endOffset
    );
  }
  get length() {
    return Math.max(0, this.endOffset - this.startOffset + 1);
  }
  getStream() {
    return this.fileMedia.createReadStream({
      start: this.startOffset,
      end: this.endOffset,
    });
  }
};
