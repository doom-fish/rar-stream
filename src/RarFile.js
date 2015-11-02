import fs from 'fs';
import stream from 'stream';
const ARCHIVE_END_PADDING = 7;

export default class RarFile {
  constructor(options, synchronous){
    if(typeof options === 'string'){
      this.path = options;

      let nameParts = options.split('/');
      this.name = nameParts[nameParts.length - 1];
      this.length = fs.statSync(this.path)["size"] - ARCHIVE_END_PADDING;
      if(synchronous){
        this.stream = new stream.PassThrough();
        this.stream.end(new Buffer(fs.readFileSync(this.path)));
      }else{
        this.stream = fs.createReadStream(this.path);
      }
    }else{
      Object.assign(this, options);
    }
  }
}