import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import sinon from 'sinon';

chai.use(chaiAsPromised);
let expect = chai.expect;
let assert = chai.assert;

import {Buffer} from 'buffer';

import ArchiveHeadParser from '../../src/parsing/ArchiveHeadParser';

const ArchiveHeadData = new Buffer('CF907300000D00000000000000D900', 'hex');


describe('ArchiveHeadParser', () => {
  let instance;
  let archiveHeader;
  beforeEach(() => {
    instance = new ArchiveHeadParser(ArchiveHeadData);
    archiveHeader = instance.parse();
  });
  describe('#constructor', () => {
    it('should be constructable', () => {
      expect(instance).to.be.an.instanceof(ArchiveHeadParser);
    });
    it('should take a stream as constructor parameter', () => {
      expect(() => new ArchiveHeadParser()).to.throw(/Invalid Arguments/);  
    });
  });
  describe('#parse', () => {
    it('should correctly parse crc correctly', () => {
      archiveHeader.crc.should.equal(0x90CF);
    });
    it('should correctly parse type', () => {
      archiveHeader.type.should.equal(0x73);
    });
    it('should correctly parse flags', () => {
      archiveHeader.flags.should.equal(0x0);
    });
    it('should parse size correctly', () => {
      archiveHeader.size.should.equal(0x0D);
    });
    it('should parse reserved1 and reserved2 properly', () => {
      archiveHeader.reserved1.should.equal(0x0);
      archiveHeader.reserved2.should.equal(0x00D90000);
    });
    it('should parse flags correctly', () => {
      archiveHeader.hasVolumeAttributes.should.be.false;
      archiveHeader.hasVolumeAttributes.should.be.false;
      archiveHeader.hasComment.should.be.false;
      archiveHeader.isLocked.should.be.false;
      archiveHeader.hasSolidAttributes.should.be.false;
      archiveHeader.isNewNameScheme.should.be.false;
      archiveHeader.hasAuthInfo.should.be.false;
      archiveHeader.hasRecovery.should.be.false;
      archiveHeader.isBlockEncoded.should.be.false;
      archiveHeader.isFirstVolume.should.be.false;
    });
  });
});