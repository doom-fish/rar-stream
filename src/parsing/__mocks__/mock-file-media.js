const { bufferFromString } = require('./utils');
const MockFileStream = require('./mock-file-stream');
module.exports = class MockFileMedia {
  constructor(stringData, name = 'MockStream') {
    this.buffer = bufferFromString(stringData.replace(/\s/g, ''));
    const byteLength = stringData.length;
    this.name = name;
    this.length = byteLength / 2;
  }
  createReadStream(options) {
    return new MockFileStream(this.buffer, options);
  }
};
