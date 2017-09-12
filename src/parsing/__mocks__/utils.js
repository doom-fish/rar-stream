const { mockStreamFromString } = require('./mock-buffer-stream');

module.exports.bind = (Parser, size) => ({
    newParser(binaryStr) {
        return new Parser(mockStreamFromString(binaryStr, { size }));
    },
    parseHeader(field, binaryStr) {
        return new Parser(
            mockStreamFromString(binaryStr, { size: size })
        ).parse()[field];
    },
});

module.exports.newPadding = count =>
    Array(count * 2)
        .fill('0')
        .join('');

module.exports.hammingWeight = num => {
    num = num - ((num >> 1) & 0x55555555);
    num = (num & 0x33333333) + ((num >> 2) & 0x33333333);
    return (((num + (num >> 4)) & 0xf0f0f0f) * 0x1010101) >> 24;
};

module.exports.btoh = binary => {
    const str = binary.toString(16);
    return str.length % 2 !== 0 ? '0' + str : str;
};
