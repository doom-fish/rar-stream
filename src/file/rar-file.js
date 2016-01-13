import {AbstractFileMedia} from "./index";
import {
  MarkerHeaderParser,
  ArchiveHeaderParser,
  FileHeaderParser,
  TerminatorHeaderParser
} from "../parsing";

export default class RarFile {
  constructor(fileMedia) {
    if (!(fileMedia instanceof AbstractFileMedia)) {
      throw Error("Invalid Arguments, expected fileMedia to be an of AbstractFileMedia");
    }
    this._files = new Set();
    this._offset = 0;
    this._readMarkerHeader()
        .then(() => this._readArchiveHeader());
  }
  _readMarkerHeader() {
    return this._createReadStream(this._offset, MarkerHeaderParser.size)
               .then(MarkerHeaderParser.parse)

  }
}
