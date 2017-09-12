const { mockStreamFromString } = require('./mock-buffer-stream');

module.exports = class MockFileMedia {
    constructor(stringData, name = 'MockStream') {
        this.stringData = stringData.replace(/\s/g, '');
        const byteLength = stringData.length;
        this.name = name;
        this.size = byteLength / 2;
    }
    async createReadStream({ start, end }) {
        const stream = mockStreamFromString(this.stringData, {
            start,
            end,
        });

        return new Promise((resolve, reject) => {
            stream.once('readable', () => resolve(stream));
            stream.on('error', reject);
        });
    }
};
