// @flow
import {Readable} from 'stream';

export type FileInterval = {start: number, end: number};

export type FileInfo = {
    name: string,
    size: number,
    createReadStream: (interval: FileInterval) => Readable
}

export default class FileMedia {
    _createReadStream: (interval: FileInterval) => Readable;
    _name: string;
    _size: number;
    constructor (fileInfo: FileInfo) {
        this._createReadStream = (interval: FileInterval) => fileInfo.createReadStream(interval);
        this._name = fileInfo.name;
        this._size = fileInfo.size;
    }
    get name () : string {
        return this._name;
    }
    get size () : number {
        return this._size;
    }
  
    createReadStreamSync (interval: FileInterval): Readable {
        const {start, end} = interval;
        if (start > end) {
            throw Error('Invalid Arguments, start offset can not be greater than end offset');
        }
        return this._createReadStream({start, end: end});
    }

    createReadStream (interval: FileInterval): Promise<Readable> {
        const {start, end} = interval;
        if (start > end) {
            throw Error('Invalid Arguments, start offset can not be greater than end offset');
        }
        let stream = this._createReadStream({start, end: end});

        return new Promise((resolve, reject) => {
            stream.on('readable', () => resolve(stream));
            stream.on('error', (error) => reject(error));
        });
    }
}
