import path from 'path';

import chai from 'chai';
let expect = chai.expect;
chai.should();

import RarHeaderParser from '../src/RarHeaderParser';
import RarFile from '../src/RarFile';

let singleFile = path.resolve(__dirname, 'binary-data/single-short-filename.rar');
let multipleFiles = path.resolve(__dirname, 'binary-data/multiple-files.rar');


describe('RarHeaderParser', () => {
  describe('#constructor', () => {
    it('should take a RarFile as argument', () => { 
      // expect(() => new RarHeaderParser()).to.throw(/Invalid Arguments/);
    });
  });
  describe('#parseMultipleFiles', () => {
    let instance;
    beforeEach(() => {
      instance = new RarHeaderParser(new RarFile(multipleFiles, true));
    });
    it('should have 11 files', () => {
      console.log([...instance].length);
    });
  });
  // describe('#parseSingleFile', () => {
  //   let instance;
  //   beforeEach(() => {
  //     instance = new RarHeaderParser(new RarFile(singleFile, true));
  //   })
  //   it('should parse markerHead data', () => {
  //     expect(instance.markerHeader).to.be.eql({
  //       add_size: 0,
  //       crc: 24914,
  //       flags: 6689,
  //       head_size: 7,
  //       head_type: 114
  //     });
  //   });
  //   it('should parse archiveHead data', () => {
  //     expect(instance.archiveHeader).to.be.eql({
  //       "auth_info": false,
  //       "block_head_enc": false,
  //       "comment": false,
  //       "crc": -28465,
  //       "first_volume": false,
  //       "flags": 0,
  //       "has_recovery": false,
  //       "head_size": 13,
  //       "head_type": 115,
  //       "lock": false,
  //       "new_namescheme": false,
  //       "reserved1": 0,
  //       "reserved2": 0,
  //       "solid_attr": false,
  //       "volume_attr": false
  //     });
  //   });
  //   it('should parse fileHeader data', () => {
  //     // expect([...instance][0]).to.be.eql({
  //     //   "attr": 33261,
  //     //   "continue_next": false,
  //     //   "continue_prev": false,
  //     //   "fileName": "rar",
  //     //   "crc": 12688,
  //     //   "encrypted": false,
  //     //   "extended_time": true,
  //     //   "file_crc": 1775311619,
  //     //   "flags": -28544,
  //     //   "has_comment": false,
  //     //   "has_high_size": false,
  //     //   "has_salt": false,
  //     //   "head_size": 37,
  //     //   "head_type": 116,
  //     //   "host": 3,
  //     //   "info_from_prev": false,
  //     //   "method": 51,
  //     //   "name_size": 3,
  //     //   "name_special": false,
  //     //   "old_version": false,
  //     //   "size": 211867,
  //     //   "timestamp": 1197169034,
  //     //   "unp_size": 570952,
  //     //   "version": 29
  //     // });
  //   });
  // })
});
