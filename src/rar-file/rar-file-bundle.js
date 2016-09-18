//@flow
import FileMedia from '../file-media/file-media'

const RXX_EXTENSION = /\.R(\d\d)$|.RAR$/i;
const RAR_EXTENSION = /.RAR$/i;
const PARTXX_RAR_EXTENSION = /.PART(\d\d).RAR/i;

export default class RarFileBundle {
  _fileMedias: FileMedia[];
  _extensionType: string;
  _length: number;

  constructor(...fileMedias: FileMedia[]) {
    this._fileMedias = fileMedias;
    if(this._fileMedias.length > 0) {
      this._resolveFileExtension();
      this.filter();
      this.sort();
    }
  }
  filter() {
    if (this._extensionType === 'rxx') {

      this._fileMedias = this._fileMedias.filter(
        (file) => (file.name && file.name.match(RXX_EXTENSION))
      );
    }else {
      this._fileMedias = this._fileMedias.filter((file) => file.name.match(PARTXX_RAR_EXTENSION));
    }
  }
  sort() {
    if (this._extensionType === 'rxx') {
      this._sortRxx();
    }else {
      this._sortPartxx();
    }
  }
  _resolveFileExtension() {
    let anyPartXXTypes = this._fileMedias.filter((file) => (
      file.name && file.name.match(PARTXX_RAR_EXTENSION))
    );

    if (anyPartXXTypes.length > 0) {
      this._extensionType = 'partxx';
    }else {
      this._extensionType = 'rxx';
    }
  }
  _sortPartxx() {
    this._fileMedias.sort((first, second) => {
      let firstMatch = first.name.match(PARTXX_RAR_EXTENSION);
      let secondMatch = second.name.match(PARTXX_RAR_EXTENSION);
      let firstNumber = +(firstMatch && firstMatch[1] || 0);
      let secondNumber = +(secondMatch && secondMatch[1] || 0);
      return firstNumber - secondNumber;
    });
  }
  _sortRxx() {
    this._fileMedias.sort((first, second) => {
      if (first.name.match(RAR_EXTENSION)) {
        return -1;
      } else if (second.name.match(RAR_EXTENSION)) {
        return 1;
      } else {
        let firstMatch = first.name.match(RXX_EXTENSION);
        let secondMatch = second.name.match(RXX_EXTENSION);
        let firstNumber = +(firstMatch && firstMatch[1] || 0);
        let secondNumber = +(secondMatch && secondMatch[1] || 0);
        return firstNumber - secondNumber;
      }
    });
  }
  get length(): number {
    return this._fileMedias.length;
  }
  get fileNames() :  string[] {
    return this._fileMedias.map(file => file.name);
  }
  get files() : FileMedia[]{
    return this._fileMedias;
  }
}
