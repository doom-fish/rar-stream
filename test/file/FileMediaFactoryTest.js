import chai from 'chai';
import path from 'path';

let expect = chai.expect;
chai.should();

import FileMediaFactory from '../../src/file/FileMediaFactory';
import FileMediaTypes from '../../src/file/FileMediaTypes';
import TorrentFileMedia from '../../src/file/TorrentFileMedia';
import LocalFileMedia from '../../src/file/LocalFileMedia';

describe('FileMediaFactory', () => {
  describe("#resolve", () => {
    it('should throw if called without arguments', () => {
      expect(() => FileMediaFactory.createInstance()).to.throw(/Invalid Arguments/);
    });
    it('should return a TorrentFileMedia if file type torrent is passed', () => {
      let instance = FileMediaFactory.createInstance(FileMediaTypes.TORRENT, {});
      expect(instance).to.be.an.instanceOf(TorrentFileMedia);
    });
    it('should return a LocalFileMedia if file type local is passed', () => {
      let instance = FileMediaFactory.createInstance(FileMediaTypes.LOCAL, path.resolve(__dirname, './FileMediaFactoryTest.js'));
      expect(instance).to.be.an.instanceOf(LocalFileMedia);
    });
  });

});
