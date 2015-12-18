import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import sinon from 'sinon';

chai.use(chaiAsPromised);
let expect = chai.expect;
let assert = chai.assert;

import {Buffer} from 'buffer';

import MarkerHeadParser from '../../src/parsing/MarkerHeadParser';

const MarkerHeadData = new Buffer('526172211A0700', 'hex');


describe('MarkerHeadParser', () => {
  let instance;
  let markerHeader;
  beforeEach(() => {
    instance = new MarkerHeadParser(MarkerHeadData);
    markerHeader = instance.parse();
  });
  describe('#constructor', () => {
    it('should be constructable', () => {
      expect(instance).to.be.an.instanceof(MarkerHeadParser);
    });
    it('should take a stream as constructor parameter', () => {
      expect(() => new MarkerHeadParser()).to.throw(/Invalid Arguments/);  
    });
  });
  describe('#parse', () => {
    it('should correctly parse crc correctly', () => {
      markerHeader.crc.should.equal(0x6152);

      let invalidInstance = new MarkerHeadParser(new Buffer('526272211A0700', 'hex'));
      let invalidMarkerHead = invalidInstance.parse();
      invalidMarkerHead.crc.should.not.equal(0x6152);
    });
    it('should correctly parse type correctly', () => {
      markerHeader.type.should.equal(0x72);
      
      let invalidInstance = new MarkerHeadParser(new Buffer('526275211A0700', 'hex'));
      let invalidMarkerHead = invalidInstance.parse();
      invalidMarkerHead.type.should.not.equal(0x72);
    });
    it('should parse flags correctly', () => {
      markerHeader.flags.should.equal(0x1A21);
      
      let invalidInstance = new MarkerHeadParser(new Buffer('23462346234623462346', 'hex'));
      let invalidMarkerHead = invalidInstance.parse();
      invalidMarkerHead.flags.should.not.equal(0x1A21);
    });
    it('should parse size properly', () => {
      markerHeader.size.should.equal(0x07);
      
      let invalidInstance = new MarkerHeadParser(new Buffer('23462346234623462346', 'hex'));
      let invalidMarkerHead = invalidInstance.parse();
      invalidMarkerHead.flags.should.not.equal(0x07);
    });
    it('should parse add_size properly', () => {
      let addSizeInstance = new MarkerHeadParser(new Buffer('526172219A070001000000', 'hex'));
      let addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x08);

      addSizeInstance = new MarkerHeadParser(new Buffer('526172219A070009000000', 'hex'));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x10);

      addSizeInstance = new MarkerHeadParser(new Buffer('526172219A07000A000000', 'hex'));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x11);

      addSizeInstance = new MarkerHeadParser(new Buffer('526172219A0700F8FFFFFF', 'hex'));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0xFFFFFFFF);
    });
  });
});