import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {mockStreamFromString} from "../mocks/mock-buffer-stream";

chai.use(chaiAsPromised);
let expect = chai.expect;

import ArchiveHeaderParser from "../../src/parsing/archive-header-parser";

describe("ArchiveHeaderParser", () => {
  let instance;
  let archiveHeader;

  beforeEach(() => {
    instance = new ArchiveHeaderParser(mockStreamFromString("CF907300000D00000000000000"));
    archiveHeader = instance.parse();
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      expect(instance).to.be.an.instanceof(ArchiveHeaderParser);
    });
    it("should take a stream as constructor parameter", () => {
      expect(() => new ArchiveHeaderParser()).to.throw(/Invalid Arguments/);
    });
  });
  describe("#bytesToRead", () => {
    it("should return a bytesToRead constant of 13", () => {
      instance.bytesToRead.should.be.eql(13);
    });
  });
  describe("#parse", () => {
    it("should correctly parse crc correctly", () => {
      archiveHeader.crc.should.equal(0x90CF);
    });
    it("should correctly parse type", () => {
      archiveHeader.type.should.equal(0x73);
    });
    it("should correctly parse flags", () => {
      archiveHeader.flags.should.equal(0x0);
    });
    it("should parse size correctly", () => {
      archiveHeader.size.should.equal(0x0D);
    });
    it("should parse reserved1 and reserved2 properly", () => {
      archiveHeader.reserved1.should.equal(0x0);
      archiveHeader.reserved2.should.equal(0x0);
    });
    it("should parse flags correctly", () => {
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
