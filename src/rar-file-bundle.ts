const RXX_EXTENSION = /\.R(\d\d)$|.RAR$/i;
const RAR_EXTENSION = /.RAR$/i;
const PARTXX_RAR_EXTENSION = /.PART(\d\d).RAR/i;
import { IFileMedia } from "./interfaces.js";

const isPartXXExtension = (fileMedias: IFileMedia[] = []) => {
  let anyPartXXTypes = fileMedias.filter(
    (file) => file.name && file.name.match(PARTXX_RAR_EXTENSION)
  );

  if (anyPartXXTypes.length > 0) {
    return true;
  } else {
    return false;
  }
};

class NumericRarFileBundle {
  constructor(private fileMedias: IFileMedia[] = []) {
    if (this.fileMedias.length > 0) {
      this.filter();
      this.sort();
    }
  }
  filter() {
    this.fileMedias = this.fileMedias.filter(
      (file) => file.name && file.name.match(RXX_EXTENSION)
    );
  }
  sort() {
    this.fileMedias.sort((first, second) => {
      if (first.name.match(RAR_EXTENSION)) {
        return -1;
      } else if (second.name.match(RAR_EXTENSION)) {
        return 1;
      } else {
        const firstMatch = first.name.match(RXX_EXTENSION);
        const secondMatch = second.name.match(RXX_EXTENSION);
        const firstNumber = +((firstMatch && firstMatch[1]) || 0);
        const secondNumber = +((secondMatch && secondMatch[1]) || 0);
        return firstNumber - secondNumber;
      }
    });
  }

  get length() {
    return this.fileMedias.length;
  }
  get fileNames() {
    return this.fileMedias.map((file) => file.name);
  }
  get files() {
    return this.fileMedias;
  }
}

class PartXXRarBundle {
  constructor(private fileMedias: IFileMedia[] = []) {
    if (this.fileMedias.length > 0) {
      this.filter();
      this.sort();
    }
  }
  filter() {
    this.fileMedias = this.fileMedias.filter((file) =>
      file.name.match(PARTXX_RAR_EXTENSION)
    );
  }
  sort() {
    this.fileMedias.sort((first, second) => {
      const firstMatch = first.name.match(PARTXX_RAR_EXTENSION);
      const secondMatch = second.name.match(PARTXX_RAR_EXTENSION);
      const firstNumber = +((firstMatch && firstMatch[1]) || 0);
      const secondNumber = +((secondMatch && secondMatch[1]) || 0);
      return firstNumber - secondNumber;
    });
  }

  get length() {
    return this.fileMedias.length;
  }
  get fileNames() {
    return this.fileMedias.map((file) => file.name);
  }
  get files() {
    return this.fileMedias;
  }
}

export type RarFileBundle = PartXXRarBundle | NumericRarFileBundle;
export const makeRarFileBundle = (
  fileMedias: IFileMedia[] = []
): RarFileBundle => {
  return isPartXXExtension(fileMedias)
    ? new PartXXRarBundle(fileMedias)
    : new NumericRarFileBundle(fileMedias);
};
