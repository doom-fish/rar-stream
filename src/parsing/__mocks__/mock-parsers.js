// @flow
type Empty = {};
export class MockEmptyParser {
    parse(): Empty {
        return {};
    }
}
type MockFileHeader = { name: string };
export class MockFileHeaderParser {
    parse(): MockFileHeader {
        return {
            name: 'test'
        };
    }
}
