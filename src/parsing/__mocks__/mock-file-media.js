// @flow
import { mockStreamFromString } from './mock-buffer-stream';
import FileMedia from '../../file-media/file-media';

export default class MockFileMedia extends FileMedia {
    constructor(stringData: string, name: ?string) {
        stringData = stringData.replace(/\s/g, '');
        const byteLength = stringData.length;
        super({
            name: name || 'MockStream',
            size: byteLength / 2,
            createReadStream: ({ start, end }) => {
                return mockStreamFromString(stringData, {
                    start,
                    end,
                    byteLength
                });
            }
        });
    }
}
