//@flow
import {Readable} from 'stream';
import RarFileBundle from './rar-file-bundle';

export default class RarStream extends Readable {
  constructor(rarFileBundle: RarFileBundle, options: Object) {
    super(options);

  }
  _read(){

  }
}
