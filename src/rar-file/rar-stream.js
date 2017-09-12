const { Readable } = require('stream');

module.exports = class RarStream extends Readable {
    constructor(rarFileChunks, options) {
        super(options);
        this.rarFileChunks = rarFileChunks;
    }
    pushData(data) {
        if (!this.push(data)) {
            this.stream.pause();
        }
    }
    get isStarted() {
        return !!this.stream;
    }
    async next() {
        const chunk = this.rarFileChunks.shift();
        if (!chunk) {
            this.push(null);
        } else {
            this.stream = await chunk.getStream();
            this.stream.on('data', data => this.pushData(data));
            this.stream.on('end', () => this.next());
        }
    }
    async _read() {
        if (!this.isStarted) {
            await this.next();
            this.stream.resume();
        } else {
            this.stream.resume();
        }
    }
};
