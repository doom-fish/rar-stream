//
module.exports.MockEmptyParser = class {
    parse() {
        return {};
    }
};
module.exports.MockFileHeaderParser = class {
    parse() {
        return {
            name: 'test',
        };
    }
};
