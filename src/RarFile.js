import fs from 'fs';
const ARCHIVE_END_PADDING = 7;
const TYPES = {
  LOCAL: 'LOCAL',
  TORRENT: 'TORRENT'
};
export default class RarFile {
  constructor(options){
    if(!options){
      throw new Error("Invalid Arguments, options need to be either a string or object");
    }
    if(typeof options === 'string'){
      this.path = options;
      this.type = TYPES.LOCAL;
      let nameParts = options.split('/');
      this.name = nameParts[nameParts.length - 1];
      this.size = fs.statSync(this.path)["size"] - ARCHIVE_END_PADDING;
  
    }else{
      Object.assign(this, options);
    }
  }
  read(start, end){
    switch (this.type){
      case TYPES.LOCAL: {
        let stream = fs.createReadStream(this.path, {start: start, end: end});
        return new Promise((resolve, reject) => {
          stream.on('readable', () => resolve(stream));
          stream.on('error', (error) => reject(error));
        })
      }
      case TYPES.TORRENT: {
        let stream = this.torrentFile.createReadStream( {start: start, end: end});
        return new Promise((resolve, reject) => {
          stream.on('readable', () => resolve(stream));
          stream.on('error', (error) => reject(error));
        })
      }
    }
  }
}