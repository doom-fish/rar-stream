import { Readable } from "stream";
export class InnerFileStream extends Readable {
    rarFileChunks;
    stream;
    constructor(rarFileChunks, options) {
        super(options);
        this.rarFileChunks = rarFileChunks;
    }
    pushData(data) {
        if (!this.push(data)) {
            this.stream?.pause();
        }
    }
    get isStarted() {
        return !!this.stream;
    }
    next() {
        const chunk = this.rarFileChunks.shift();
        if (!chunk) {
            this.push(null);
        }
        else {
            this.stream = chunk.getStream();
            this.stream?.on("data", (data) => this.pushData(data));
            this.stream?.on("end", () => this.next());
        }
    }
    _read() {
        if (!this.isStarted) {
            this.next();
        }
        else {
            this.stream?.resume();
        }
    }
}
