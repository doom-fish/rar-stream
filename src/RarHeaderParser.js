import binary from 'binary';
import rarFileType from './RarFile';

const readMarkerHeader = Symbol();
const readArchiveHeader = Symbol();
const readFileHeader = Symbol();
const readFileName = Symbol();
const stream = Symbol();
const offset = Symbol();
const rarFile = Symbol();

const MARKER_HEAD_SIZE = 11;
const ARCHIVE_HEAD_SIZE = 13;
const FILE_HEAD_SIZE = 300;

export default class RarHeaderParser {
  constructor(rarFileInstance){
    if(!(rarFileInstance instanceof rarFileType)){
      throw new Error("Invalid Arguments, rarFileInstance need to be a RarFile, was: ", typeof rarFileInstance);
    }

    this[rarFile] = rarFileInstance;

    this.files = [];
    this[offset] = 0;
  }
  parseMarkerHead(){
     return this[rarFile].read(0, MARKER_HEAD_SIZE)
                .then(stream => {
                  this.markerHeader = this[readMarkerHeader](stream);
                  this[offset] += this.markerHeader.head_size;
                });
  }
  parseArchiveHead(){
    return this[rarFile].read(this[offset], this[offset] + ARCHIVE_HEAD_SIZE)
                       .then(stream => {
                          this.archiveHeader = this[readArchiveHeader](stream);
                          this[offset] += this.archiveHeader.head_size;
                       });
  }
  parseFiles(){
    return this[rarFile].read(this[offset], this[offset] + FILE_HEAD_SIZE)
               .then(stream => {
                  let file = this[readFileHeader](stream);
                  this.files.push(file);
                  this[offset] += file.size + file.head_size;
 
                  if(this[offset] < this[rarFile].size){
                    return this.parseFiles();
                  } else {
                    return this;
                }
              });         
  }
  parse(){
    return this.parseMarkerHead()
                .then(() => this.parseArchiveHead())
                .then(() => this.parseFiles());
  }
  *[Symbol.iterator] (){
    yield* this.files;
  }
  [readMarkerHeader](stream) {
    return binary.parse(stream.read(7))
      .word16ls("crc")
      .word8ls("head_type")
      .word16ls("flags")
      .word16ls("head_size")
      .tap((vars) => {
        if((vars.flags & 0x8000) !== 0){
          vars.add_size = binary.parse(stream.read(4))
                                .word32ls("add_size")
                                .vars
                                .add_size;
        }else{
          vars.add_size = 0;
        }
      })
      .vars;
  }
  [readArchiveHeader](stream) {
    return binary.parse(stream.read(13))
      .word16ls("crc")
      .word8ls("head_type")
      .word16ls("flags")
      .word16ls("head_size")
      .word16ls("reserved1")
      .word32ls("reserved2")
      .tap((vars) => {
        vars.volume_attr = (vars.flags & 0x0001) !== 0;
        vars.comment = (vars.flags & 0x0002) !== 0;
        vars.lock = (vars.flags & 0x0004) !== 0;
        vars.solid_attr = (vars.flags & 0x0008) !== 0;
        vars.new_namescheme = (vars.flags & 0x00010) !== 0;
        vars.auth_info = (vars.flags & 0x0020) !== 0;
        vars.has_recovery = (vars.flags & 0x0040) !== 0;
        vars.block_head_enc = (vars.flags & 0x0080) !== 0;
        vars.first_volume = (vars.flags & 0x0100) !== 0;
      }).vars;
  }
  [readFileHeader](stream) {
    return binary.parse(stream.read(32))
      .word16ls("crc")
      .word8ls("head_type")
      .word16ls("flags")
      .word16ls("head_size")
      .word32ls("size")
      .word32ls("unp_size")
      .word8ls("host")
      .word32ls("file_crc")
      .word32ls("timestamp")
      .word8ls("version")
      .word8ls("method")
      .word16ls("name_size")
      .word32ls("attr")
      .tap((vars) => {
        vars.continue_prev = (vars.flags & 0x01) !== 0;
        vars.continue_next = (vars.flags & 0x02) !== 0;
        vars.encrypted = (vars.flags & 0x04) !== 0;
        vars.has_comment = (vars.flags & 0x08) !== 0;
        vars.info_from_prev = (vars.flags & 0x10) !== 0;
        vars.has_high_size = (vars.flags & 0x100) !== 0;
        vars.name_special = (vars.flags & 0x200) !== 0;
        vars.has_salt = (vars.flags & 0x400) !== 0;
        vars.old_version = (vars.flags & 0x800) !== 0;
        vars.extended_time = (vars.flags & 0x1000) !== 0;
        if (vars.has_high_size) {
           binary.parse(stream.read(8))
            .word32ls("high_pack_size")
            .word32ls("high_unp_size")
            .tap((high_size_vars) => {
              vars.size = high_size_vars.high_pack_size * 0x100000000 + vars.size;
              vars.unp_size = high_size_vars.high_unp_size * 0x100000000 + vars.unp_size;
            });
        }
        vars.name = stream.read(vars.name_size).toString();
      }).vars;
  }
}