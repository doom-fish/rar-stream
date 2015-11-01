import fs from 'fs';
import path from 'path';
import stream from 'stream';
import chai from 'chai';
let expect = chai.expect;
chai.should();

import RarHeaderParser from '../RarHeaderParser';

let singleShortFileName = path.resolve(__dirname, 'binary-data/single-short-filename.rar');
let singleShortFilenameBuffer = new Buffer(fs.readFileSync(singleShortFileName));


describe('RarHeaderParser', () => {
  describe('#constructor', () => {
    it('should be a constructable', () => {
      let instance = new RarHeaderParser();
      expect(instance).to.be.an.instanceOf(RarHeaderParser);
    });
  });
  describe('#parseSingleShortFilename', () => {
    let instance;
    beforeEach(() => {
      let singleShortFileManeStream = new stream.PassThrough();
      singleShortFileManeStream.end(singleShortFilenameBuffer);
      instance = new RarHeaderParser(singleShortFileManeStream);
      instance.parse();
    })
    it('should parse markerHead data', () => {
      expect(instance.markerHeader).to.be.eql({
        add_size: 0,
        crc: 24914,
        flags: 6689,
        head_size: 7,
        head_type: 114
      });
    });
    it('should parse archiveHead data', () => {
      expect(instance.archiveHeader).to.be.eql({
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
    it('should parse fileHeader data', () => {
      expect(instance.fileHeader).to.be.eql({
        "attr": 33261,
        "continue_next": false,
        "continue_prev": false,
        "crc": 12688,
        "encrypted": false,
        "extended_time": true,
        "file_crc": 1775311619,
        "flags": -28544,
        "has_comment": false,
        "has_high_size": false,
        "has_salt": false,
        "head_size": 37,
        "head_type": 116,
        "host": 3,
        "info_from_prev": false,
        "method": 51,
        "name_size": 3,
        "name_special": false,
        "old_version": false,
        "size": 211867,
        "timestamp": 1197169034,
        "unp_size": 570952,
        "version": 29
      });
    });
    it('should parse a correct filename', () => {
      expect(instance.fileName).to.be.eql('rar');
    })
  })
});
