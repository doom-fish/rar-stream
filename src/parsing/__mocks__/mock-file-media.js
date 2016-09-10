import {mockStreamFromString} from './mock-buffer-stream';
import {FileMedia} from '../../src/file';

export default class MockFileMedia extends FileMedia {
  constructor(stringData, size) {
    super({
      name: 'MockStream',
      size: stringData.length,
      createReadStream: (start, end) => {
        return mockStreamFromString(stringData, {start, end, size});
      }
    });
  }
}
