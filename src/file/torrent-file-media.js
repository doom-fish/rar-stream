import FileMedia from "./file-media";

export default class TorrentFileMedia extends FileMedia {
  constructor(fileInfo) {
    if (!fileInfo) {
      throw new Error("Invalid Arguments, fileInfo need to be passed to the constructor");
    }
    fileInfo.size = fileInfo.length;
    super(fileInfo);
  }
}
