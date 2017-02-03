'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});

var _fileMedia = require('../file-media/file-media');

var _fileMedia2 = _interopRequireDefault(_fileMedia);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

const RXX_EXTENSION = /\.R(\d\d)$|.RAR$/i;

const RAR_EXTENSION = /.RAR$/i;
const PARTXX_RAR_EXTENSION = /.PART(\d\d).RAR/i;

class RarFileBundle {

    constructor(...fileMedias) {
        this._fileMedias = fileMedias;
        if (this._fileMedias.length > 0) {
            this._resolveFileExtension();
            this.filter();
            this.sort();
        }
    }
    filter() {
        if (this._extensionType === 'rxx') {
            this._fileMedias = this._fileMedias.filter(file => file.name && file.name.match(RXX_EXTENSION));
        } else {
            this._fileMedias = this._fileMedias.filter(file => file.name.match(PARTXX_RAR_EXTENSION));
        }
    }
    sort() {
        if (this._extensionType === 'rxx') {
            this._sortRxx();
        } else {
            this._sortPartxx();
        }
    }
    _resolveFileExtension() {
        let anyPartXXTypes = this._fileMedias.filter(file => file.name && file.name.match(PARTXX_RAR_EXTENSION));

        if (anyPartXXTypes.length > 0) {
            this._extensionType = 'partxx';
        } else {
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
    get length() {
        return this._fileMedias.length;
    }
    get fileNames() {
        return this._fileMedias.map(file => file.name);
    }
    get files() {
        return this._fileMedias;
    }
}
exports.default = RarFileBundle;