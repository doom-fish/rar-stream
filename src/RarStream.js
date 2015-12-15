import {Readable} from 'stream';
import util from 'util';

class RarStream{
  constructor(options){
    Readable.call(this, options);
  }
}
util.inherits(RarStream, Readable);

export default RarStream;