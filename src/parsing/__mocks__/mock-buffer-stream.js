const { Readable } = require('stream');

class MockFileStream extends Readable {
    constructor(object, options) {
        super(options);
        this.options = options;
        this.object = object;
    }
    _read() {
        if (
            !!this.object &&
            typeof this.options.start === 'number' &&
            typeof this.options.end === 'number'
        ) {
            const buffer = this.object.slice(
                this.options.start,
                this.options.end
            );
            this.push(buffer);
            this.object = null;
        } else {
            this.push(this.object);
            this.object = null;
        }
    }
}

MockFileStream.mockStreamFromString = (str, options = {}, variant = 'hex') => {
    if (options.size) {
        let padding = Math.abs(+options.size - str.length / 2);
        str += Array(padding)
            .fill()
            .map(() => '00')
            .join('');
    }
    return new MockFileStream(new Buffer(str, variant), options);
};

module.exports = MockFileStream;
