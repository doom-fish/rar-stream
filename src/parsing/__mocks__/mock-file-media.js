//@flow
import {mockStreamFromString} from './mock-buffer-stream';
import FileMedia from '../../file-media/file-media';

export default class MockFileMedia extends FileMedia {
  constructor(stringData: string) {
    const byteLength = stringData.replace(/\s/g, '').length;
    super({
      name: 'MockStream',
      size: byteLength,
      createReadStream: (start, end) => {
        return mockStreamFromString(stringData, {start, end, byteLength});
      }
    });
  }
}
