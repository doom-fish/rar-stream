// @flow
import { Readable } from 'stream';
type Options = readableStreamOptions;

export class MockFileStream extends Readable {
    _object: ?Buffer;
    _options: Options;
    constructor(object: ?Buffer, options: Options) {
        super(options);
        this._options = options;
        this._object = object;
    }
    _read() {
        if (
            !!this._object &&
                typeof this._options.start === 'number' &&
                typeof this._options.end === 'number'
        ) {
            const buffer = this._object.slice(
                this._options.start,
                this._options.end
            );
            this.push(buffer);
            this._object = null;
        } else {
            this.push(this._object);
            this._object = null;
        }
    }
}

// @flow-disable
export const mockStreamFromString = (
    str: string,
    options: Options = {},
    variant: buffer$Encoding = 'hex'
) => {
    if (options.size) {
        let padding = Math.abs(+options.size - str.length / 2);
        str += Array(padding).fill().map(() => '00').join('');
    }
    return new MockFileStream(new Buffer(str, variant), options);
};
