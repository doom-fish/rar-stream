import path from 'path';
import sinon from 'sinon';
import chai from 'chai';
let expect = chai.expect;
let assert = chai.assert;

chai.should();

import RarFile from '../src/RarFile';
let singleFile = path.resolve(__dirname, 'binary-data/single-short-filename.rar');

describe('RarFile', () => {
  describe('#constructor', () => {
    it('should throw if path is options and file does not exist', () => {
      let path = '/a/b/c/d.rar';
      expect(() => new RarFile(path)).to.throw(/ENOENT: no such file or directory/);
    });
    it('should throw if no options are passed', () => {
      expect(() => new RarFile()).to.throw(/Invalid Arguments/);
    });
    it('should parse path and read file size if string is passed as options', () => {
      let instance = new RarFile(singleFile);
      expect(instance.name).to.equal('single-short-filename.rar');
      expect(instance.size).to.equal(571009);
      expect(instance.type).to.equal("LOCAL");
      expect(instance.path).to.equal(singleFile);
    });
    it('should parse options object', () =>
    {
      let instance = new RarFile({ 
                                  type: 'TORRENT', 
                                   name: 'file.rar', 
                                   size: 44
                                 });
      expect(instance.name).to.equal('file.rar');
      expect(instance.size).to.equal(44);
      expect(instance.type).to.equal("TORRENT");
    });
  });
  describe("#read", () => {
    it('should call createReadStream on torren object if file is TORRENT', () => {
      let TORRENTFile = {createReadStream(){}};
      sinon.stub(TORRENTFile, 'createReadStream');
      let instance = new RarFile({
        type: "TORRENT",
        torrentFile: TORRENTFile
      });
      instance.read(0, 24);
      assert.ok(TORRENTFile.createReadStream.calledWithMatch({start: 0, end: 24}), 'createReadStream waas not called with correct params')
    });
  });
});
