import path from 'path';
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
chai.use(chaiAsPromised);
let expect = chai.expect;
chai.should();

import RarHeaderParser from '../src/RarHeaderParser';
import RarFile from '../src/RarFile';

let singleFile = path.resolve(__dirname, 'binary-data/single-short-filename.rar');
let multipleFiles = path.resolve(__dirname, 'binary-data/multiple-files.rar');


describe('RarHeaderParser', () => {
  describe('#constructor', () => {
    it('should take a RarFile as argument', () => { 
       expect(() => new RarHeaderParser()).to.throw(/Invalid Arguments/);
    });
  });
  describe('#parse', () => {
    let singleFileInstance, multipleFilesInstance;
    beforeEach(() => {
      singleFileInstance = new RarHeaderParser(new RarFile(singleFile));
      multipleFilesInstance = new RarHeaderParser(new RarFile(multipleFiles));
    });
    it('should parse multipleFiles', () => {
      return multipleFilesInstance.parse()
                                  .then((instance) => instance.files.length)
                                  .should
                                  .eventually
                                  .equal(11);
    });
    it('should parse multipleFileNames', () => {
      return multipleFilesInstance.parse()
                                  .then((instance) => instance.files.map(file => file.name))
                                  .should
                                  .eventually
                                  .eql([
                                    "acknow.txt",
                                    "default.sfx",
                                    "license.txt",
                                    "order.htm",
                                    "rar",
                                    "rar.txt",
                                    "rarfiles.lst",
                                    "readme.txt",
                                    "singleFile.rar",
                                    "unrar",
                                    "whatsnew.txt"
                                  ]);
    });
    it('should parse mark head', () => {
      return singleFileInstance.parse()
                               .then(instance => singleFileInstance.markerHeader)
                               .should
                               .eventually
                               .eql({
                                  add_size: 0,
                                  crc: 24914,
                                  flags: 6689,
                                  head_size: 7,
                                  head_type: 114
                               });
    });
    it('should parse archive head', () => {
      return singleFileInstance.parse()
                               .then(instance => singleFileInstance.archiveHeader)
                               .should
                               .eventually
                               .eql({
                                  "auth_info": false,
                                  "block_head_enc": false,
                                  "comment": false,
                                  "crc": -28465,
                                  "first_volume": false,
                                  "flags": 0,
                                  "has_recovery": false,
                                  "head_size": 13,
                                  "head_type": 115,
                                  "lock": false,
                                  "new_namescheme": false,
                                  "reserved1": 0,
                                  "reserved2": 0,
                                  "solid_attr": false,
                                  "volume_attr": false
                               });
    });
    it('should parse single fileHeader out of single file archive', () => {
       return singleFileInstance.parse()
                               .then(instance => singleFileInstance.files[0])
                               .should
                               .eventually
                               .eql({
                                  "attr": 33261,
                                  "continue_next": false,
                                  "continue_prev": false,
                                  "crc": -416,
                                  "encrypted": false,
                                  "extended_time": true,
                                  "file_crc": 1775311619,
                                  "flags": -28640,
                                  "has_comment": false,
                                  "has_high_size": false,
                                  "has_salt": false,
                                  "head_size": 37,
                                  "head_type": 116,
                                  "host": 3,
                                  "info_from_prev": false,
                                  "method": 48,
                                  "name": "rar",
                                  "name_size": 3,
                                  "name_special": false,
                                  "old_version": false,
                                  "size": 570952,
                                  "timestamp": 1197169034,
                                  "unp_size": 570952,
                                  "version": 20
                               });
    });
  });
});
