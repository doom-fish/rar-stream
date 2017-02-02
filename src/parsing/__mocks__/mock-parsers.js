// @flow
export class MockEmptyParser {
    parse () {
        return {};
    }
}

export class MockFileHeaderParser {
    parse () {
        return {
            name: 'test'
        };
    }
}
