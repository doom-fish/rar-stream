import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import sinon from 'sinon';

chai.use(chaiAsPromised);
let expect = chai.expect;
let assert = chai.assert;

import {Buffer} from 'buffer';

import MarkHeadParser from '../../src/parsing/MarkHeadParser';

const MarkHeaderData = new Buffer('526172211A0700', 'hex');


describe('MarkHeadParser', () => {
  let instance;
  let markerHeader;
  beforeEach(() => {
    instance = new MarkHeadParser(MarkHeaderData);
    markerHeader = instance.parse();
  });
  describe('#constructor', () => {
    it('should be constructable', () => {
      expect(instance).to.be.an.instanceof(MarkHeadParser);
    });
    it('should take a stream as constructor parameter', () => {
      expect(() => new MarkHeadParser()).to.throw(/Invalid Arguments/);  
    });
  });
  describe('#parse', () => {
    it('should correctly parse crc correctly', () => {
      markerHeader.crc.should.equal(0x6152);

      let invalidInstance = new MarkHeadParser(new Buffer('526272211A0700', 'hex'));
      let invalidMarkHead = invalidInstance.parse();
      invalidMarkHead.crc.should.not.equal(0x6152);
    });
    it('should correctly parse type correctly', () => {
      markerHeader.type.should.equal(0x72);
      
      let invalidInstance = new MarkHeadParser(new Buffer('526275211A0700', 'hex'));
      let invalidMarkHead = invalidInstance.parse();
      invalidMarkHead.type.should.not.equal(0x72);
    });
    it('should parse flags correctly', () => {
      markerHeader.flags.should.equal(0x1A21);
      
      let invalidInstance = new MarkHeadParser(new Buffer('23462346234623462346', 'hex'));
      let invalidMarkHead = invalidInstance.parse();
      invalidMarkHead.flags.should.not.equal(0x1A21);
    });
    it('should parse size properly', () => {
      markerHeader.size.should.equal(0x0007);
      
      let invalidInstance = new MarkHeadParser(new Buffer('23462346234623462346', 'hex'));
      let invalidMarkHead = invalidInstance.parse();
      invalidMarkHead.flags.should.not.equal(0x0007);
    });
    it('should parse add_size properly', () => {
      let addSizeInstance = new MarkHeadParser(new Buffer('526172219A070001000000', 'hex'));
      let addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x0008);

      addSizeInstance = new MarkHeadParser(new Buffer('526172219A070009000000', 'hex'));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x0010);

      addSizeInstance = new MarkHeadParser(new Buffer('526172219A07000A000000', 'hex'));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x0011);

      addSizeInstance = new MarkHeadParser(new Buffer('526172219A0700F8FFFFFF', 'hex'));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0xFFFFFFFF);
    });
  });
});