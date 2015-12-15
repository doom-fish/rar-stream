import chai from 'chai';
let expect = chai.expect;
chai.should();

import FileMediaFactory from '../../src/file/FileMediaFactory';
import FileMediaTypes from '../../src/file/FileMediaTypes';
import TorrentFileMedia from '../../src/file/TorrentFileMedia';

describe('FileMediaFactory', () => {
  describe("#resolve", () => {
    it('should throw if called without arguments', () => {
      expect(() => FileMediaFactory.createInstance()).to.throw(/Invalid Arguments/);
    });
    it('should return a TorrentFileMedia if file type torrent is passed', () => {
      let instance = FileMediaFactory.createInstance(FileMediaTypes.TORRENT, {});
      expect(instance).to.be.an.instanceOf(TorrentFileMedia);
    });
  });

});
