// @flow
import { Readable } from 'stream';
import RarFileChunk from './rar-file-chunk';
export default class RarStream extends Readable {
    _rarFileChunks: RarFileChunk[];
    _stream: Readable;
    _index: number;
    constructor(rarFileChunks: RarFileChunk[]) {
        super();
        this._rarFileChunks = rarFileChunks;
        this._next();
    }
    pushData(data: Buffer) {
        if (!this.push(data)) {
            this._stream.pause();
        }
    }
    _next() {
        const chunk = this._rarFileChunks.shift();

        if (!chunk) {
            this.push(null);
        } else {
            this._stream = chunk.getStreamSync();
            this._stream.on('data', data => this.pushData(data));
            const self = this;
            this._stream.on('end', function() {
                self._next();
            });
        }
    }
    _read() {
        this._stream.resume();
    }
}
