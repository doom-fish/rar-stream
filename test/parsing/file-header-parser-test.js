import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {mockStreamFromString} from "../mocks/mock-buffer-stream";
chai.use(chaiAsPromised);
let expect = chai.expect;

import FileHeaderParser from "../../src/parsing/file-header-parser";

describe("FileHeaderParserTest", () => {
  let instance;
  let fileHeader;
  beforeEach(() => {
    instance = new FileHeaderParser(mockStreamFromString("D9777420902C005C" +
                                                         "1000005C10000003" +
                                                         "C5A6D2158A595B47" +
                                                         "14300A00A4810000" +
                                                         "61636B6E6F772E74" +
                                                         "787400C0", {size: 280}));
    fileHeader = instance.parse();
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      instance.should.be.an.instanceof(FileHeaderParser);
    });
    it("should take a stream as constructor parameter", () => {
      expect(() => new FileHeaderParser()).to.throw(/Invalid Arguments/);
    });
  });
  describe("#bytesToRead", () => {
    it("should return bytesToRead constant 280", () => {
      instance.bytesToRead.should.be.eql(280);
    });
  }),
  describe("#parse", () => {
    it("should parse crc properly", () => {
      fileHeader.crc.should.be.equal(0x77D9);
    });
    it("should parse type correctly", () => {
      fileHeader.type.should.be.equal(0x74);
    });
    it("should parse flags properly", () => {
      fileHeader.flags.should.be.equal(0x9020);
    });
    it("should parse headSize properly", () => {
      fileHeader.headSize.should.equal(0x002C);
    });
    it("should parse size properly", () => {
      fileHeader.size.should.equal(0x105C);
    });
    it("should parse unpackedSize properly", () => {
      fileHeader.unpackedSize.should.equal(0x105C);
    });
    it("should parse host properly", () => {
      fileHeader.host.should.equal(0x03);
    });
    it("should parse fileCrc properly", () => {
      fileHeader.fileCrc.should.equal(0x15D2A6C5);
    });
    it("should parse timestamp properly", () => {
      fileHeader.timestamp.should.equal(0x475B598A);
    });
    it("should parse version properly", () => {
      fileHeader.version.should.equal(0x14);
    });
    it("should parse method properly", () => {
      fileHeader.method.should.equal(0x30);
    });
    it("should parse nameSize properly", () => {
      fileHeader.nameSize.should.equal(0x0A);
    });
    it("should parse attributes proplery", () => {
      fileHeader.attributes.should.equal(0x000081A4);
    });
    it("should parse flags into booleans", () => {
      fileHeader.continuesFromPrevious.should.be.false;
      fileHeader.continuesInNext.should.be.false;
      fileHeader.isEncrypted.should.be.false;
      fileHeader.hasComment.should.be.false;
      fileHeader.hasInfoFromPrevious.should.be.false;
      fileHeader.hasHighSize.should.be.false;
      fileHeader.hasSpecialName.should.be.false;
      fileHeader.hasSalt.should.be.false;
      fileHeader.isOldVersion.should.be.false;
      fileHeader.hasExtendedTime.should.be.true;
    });
    it("should handle high file size", () => {
      let highFileSizeBuffer = mockStreamFromString("D97774111111115C1000005C10000003C5A6D2158A5" +
                                                   "95B4714300A00A4810000040000000400000061636B6" +
                                                   "E6F772E74787400C0", {size: 280});

      let highFileSizeHeaderParser = new FileHeaderParser(highFileSizeBuffer);
      let highFileSizeHeader = highFileSizeHeaderParser.parse();

      highFileSizeHeader.hasHighSize.should.be.true;
      highFileSizeHeader.size.should.equal(0x40000105c);
      highFileSizeHeader.unpackedSize.should.equal(0x40000105c);
    });
    it("should parse file name properly", () => {
      fileHeader.name.should.equal("acknow.txt");
    });
  });
});
