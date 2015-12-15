import chai from 'chai';
let expect = chai.expect;
chai.should();

import AbstractFileMedia from '../../src/file/AbstractFileMedia';

describe('AbstractFileMedia', () => {
  let instance;
  beforeEach(() => {
    instance = new AbstractFileMedia();
  });
  describe('#constructor', () => {
    it('should be constructable', () => {
      expect(instance).to.be.an.instanceOf(AbstractFileMedia);
    });
  });
  describe("#name", () => {
    it('should throw if accessed', () => {
      expect(() => instance.name).to.throw(/Abstract Method/);
    });
  });
  describe("#size", () => {
    it('should throw if accessed', () => {
      expect(() => instance.size).to.throw(/Abstract Method/);
    });
  });
  describe("#createReadStream", () => {
    it('should return promise', () => {
      expect(instance.createReadStream(0,0)).to.be.an.instanceOf(Promise);
    });
    it('should return an rejected promise', () => {
      return instance.createReadStream(0,0).should.eventually.be.rejectedWith(Error);
    });
  });
});
