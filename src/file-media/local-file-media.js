const { basename } = require('path');
const { statSync, createReadStream } = require('fs');

module.exports = class LocalFileMedia {
    constructor(path) {
        if (typeof path !== 'string') {
            throw new Error(
                'Invalid Arguments, path' +
                    'need to be passed to the constructor as a string'
            );
        }
        this.path = path;
        this.name = basename(path);
        this.size = statSync(path).size;
    }
    createReadStream(interval) {
        const stream = createReadStream(this.path, interval);
        return new Promise((resolve, reject) => {
            stream.once('readable', () => resolve(stream));
            stream.on('error', reject);
        });
    }
};
