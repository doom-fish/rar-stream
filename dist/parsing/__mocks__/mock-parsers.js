'use strict';

Object.defineProperty(exports, "__esModule", {
    value: true
});
class MockEmptyParser {
    parse() {
        return {};
    }
}
exports.MockEmptyParser = MockEmptyParser;
class MockFileHeaderParser {
    parse() {
        return {
            name: 'test'
        };
    }
}
exports.MockFileHeaderParser = MockFileHeaderParser;