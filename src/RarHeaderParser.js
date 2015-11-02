import binary from 'binary';
const readMarkerHeader = Symbol();
const readArchiveHeader = Symbol();
const readFileHeader = Symbol();
const readFileName = Symbol();
const rarFile = Symbol();
const offset = Symbol();

export default class RarHeaderParser {
  constructor(rarFileInstance){
    this[rarFile] = rarFileInstance;


    this.markerHeader = this[readMarkerHeader]();
    this.archiveHeader = this[readArchiveHeader]();
    this.files = new Set();

      let file = this[readFileHeader]();
      this.files.add(file);
      console.log(file)
       file = this[readFileHeader]();
       console.log(file.name);
       file = this[readFileHeader]();
       console.log(file.name);  file = this[readFileHeader]();
       console.log(file.name);  file = this[readFileHeader]();
       console.log(file.name);  file = this[readFileHeader]();
       console.log(file.name);
  
  }
  *[Symbol.iterator] (){
    yield* this.files;
  }
  [readMarkerHeader]() {
    return binary.parse(this[rarFile].stream.read(7))
      .word16ls("crc")
      .word8ls("head_type")
      .word16ls("flags")
      .word16ls("head_size")
      .tap((vars) => {
        if((vars.flags & 0x8000) !== 0){
          vars.add_size = binary.parse(this[rarFile].stream.read(4))
                                .word32ls("add_size")
                                .vars
                                .add_size;
        }else{
          vars.add_size = 0;
        }
      })
      .vars;
  }
  [readArchiveHeader]() {
    return binary.parse(this[rarFile].stream.read(13))
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
  [readFileHeader]() {
    return binary.parse(this[rarFile].stream.read(32))
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
           binary.parse(this[rarFile].stream.read(8))
            .word32ls("high_pack_size")
            .word32ls("high_unp_size")
            .tap((high_size_vars) => {

              vars.size = high_size_vars.high_pack_size * 0x100000000 + vars.size;
              vars.unp_size = high_size_vars.high_unp_size * 0x100000000 + vars.unp_size;
            });;
        }
    
        vars.name = this[rarFile].stream.read(vars.name_size).toString();
        this[rarFile].stream.read(vars.size + 2);

      }).vars;
  }
}