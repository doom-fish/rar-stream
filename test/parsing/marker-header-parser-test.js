import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {mockStreamFromString} from "../mocks/mock-buffer-stream";
chai.use(chaiAsPromised);
let expect = chai.expect;

import MarkerHeaderParser from "../../src/parsing/marker-header-parser";

describe("MarkerHeaderParser", () => {
  let instance;
  let markerHeader;
  beforeEach(() => {
    instance = new MarkerHeaderParser(mockStreamFromString("526172211A0700", {size: 11}));
    markerHeader = instance.parse();
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      expect(instance).to.be.an.instanceof(MarkerHeaderParser);
    });
    it("should take a stream as constructor parameter", () => {
      expect(() => new MarkerHeaderParser()).to.throw(/Invalid Arguments/);
    });
  });
  describe("#size", () => {
    it("should return a size constant of 7", () => {
      instance.size.should.be.eql(11);
    });
  });
  describe("#parse", () => {
    it("should correctly parse crc correctly", () => {
      markerHeader.crc.should.equal(0x6152);
      let stream = mockStreamFromString("526272211A0700", {size: 11});
      let invalidInstance = new MarkerHeaderParser(stream);
      let invalidMarkerHeader = invalidInstance.parse();
      invalidMarkerHeader.crc.should.not.equal(0x6152);
    });
    it("should correctly parse type correctly", () => {
      markerHeader.type.should.equal(0x72);
      let stream = mockStreamFromString("526275211A0700", {size: 11});
      let invalidInstance = new MarkerHeaderParser(stream);
      let invalidMarkerHeader = invalidInstance.parse();
      invalidMarkerHeader.type.should.not.equal(0x72);
    });
    it("should parse flags correctly", () => {
      markerHeader.flags.should.equal(0x1A21);
      let stream = mockStreamFromString("23462346234623", {size: 11});
      let invalidInstance = new MarkerHeaderParser(stream);
      let invalidMarkerHeader = invalidInstance.parse();
      invalidMarkerHeader.flags.should.not.equal(0x1A21);
    });
    it("should parse size properly", () => {
      markerHeader.size.should.equal(0x07);
      let stream = mockStreamFromString("23462346234623", {size: 11});
      let invalidInstance = new MarkerHeaderParser(stream);
      let invalidMarkerHeader = invalidInstance.parse();
      invalidMarkerHeader.flags.should.not.equal(0x07);
    });
    it("should parse add_size properly", () => {
      let addSizeInstance = new MarkerHeaderParser(mockStreamFromString("526172219A070001000000"));
      let addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x08);

      addSizeInstance = new MarkerHeaderParser(mockStreamFromString("526172219A070009000000"));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x10);

      addSizeInstance = new MarkerHeaderParser(mockStreamFromString("526172219A07000A000000"));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0x11);

      addSizeInstance = new MarkerHeaderParser(mockStreamFromString("526172219A0700F8FFFFFF"));
      addSizeMarker = addSizeInstance.parse();
      addSizeMarker.size.should.equal(0xFFFFFFFF);
    });
  });
});
