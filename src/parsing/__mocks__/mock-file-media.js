//@flow
import {mockStreamFromString} from './mock-buffer-stream';
import FileMedia from '../../file-media/file-media';

export default class MockFileMedia extends FileMedia {
  constructor(stringData: string, size: number) {
    super({
      name: 'MockStream',
      size: stringData.length,
      createReadStream: (start, end) => {
        return mockStreamFromString(stringData, {start, end, size});
      }
    });
  }
}
