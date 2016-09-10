//@flow
const RXX_EXTENSION = /\.R(\d\d)$|.RAR$/i;
const RAR_EXTENSION = /.RAR$/i;
const PARTXX_RAR_EXTENSION = /.PART(\d\d).RAR/i;

export default class RarFileBundle {
  _fileNames: string[];
  _extensionType: string;
  _length: number;
  constructor(fileNames: string[]) {
    if (!fileNames) {
      throw new Error('Invalid Arguments, fileNames need to be passed to the constructor');
    }
    this._fileNames = fileNames;
    this._resolveFileExtension();
    this.filter();
    this.sort();
  }
  filter() {
    if (this._extensionType === 'rxx') {
      this._fileNames = this._fileNames.filter((part) => part.match(RXX_EXTENSION));
    }else {
      this._fileNames = this._fileNames.filter((part) => part.match(PARTXX_RAR_EXTENSION));
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
    let anyPartXXTypes = this._fileNames.filter((part) => part.match(PARTXX_RAR_EXTENSION));

    if (anyPartXXTypes.length > 0) {
      this._extensionType = 'partxx';
    }else {
      this._extensionType = 'rxx';
    }
  }
  _sortPartxx() {
    this._fileNames.sort((first, second) => {
      let firstMatch = first.match(PARTXX_RAR_EXTENSION);
      let secondMatch = second.match(PARTXX_RAR_EXTENSION);
      let firstNumber = +(firstMatch && firstMatch[1] || 0);
      let secondNumber = +(secondMatch && secondMatch[1] || 0);
      return firstNumber - secondNumber;
    });
  }
  _sortRxx() {
    this._fileNames.sort((first, second) => {
      if (first.match(RAR_EXTENSION)) {
        return -1;
      } else if (second.match(RAR_EXTENSION)) {
        return 1;
      } else {
        let firstMatch = first.match(RXX_EXTENSION);
        let secondMatch = second.match(RXX_EXTENSION);
        let firstNumber = +(firstMatch && firstMatch[1] || 0);
        let secondNumber = +(secondMatch && secondMatch[1] || 0);
        return firstNumber - secondNumber;
      }
    });
  }
  get length(): number {
    return this._fileNames.length;
  }
  get fileNames() :  string[] {
    return this._fileNames;
  }
}
