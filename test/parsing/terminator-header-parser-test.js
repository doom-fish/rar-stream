import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {mockStreamFromString} from "../mocks/mock-buffer-stream";
chai.use(chaiAsPromised);

import TerminatorHeaderParser from "../../src/parsing/terminator-header-parser";
import AbstractParser from "../../src/parsing/abstract-parser.js";

describe("TerminatorHeaderParserTest", () => {
  let instance;
  let terminatorHeader;
  beforeEach(() => {
    instance = new TerminatorHeaderParser(mockStreamFromString("C43D7B00400700"));
    terminatorHeader = instance.parse();
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      instance.should.be.an.instanceof(TerminatorHeaderParser);
    });
    it("should be an instance of AbstractParser", () => {
      instance.should.be.an.instanceof(AbstractParser);
    });
  });
  describe("#size", () => {
    it("should return a size constant of 7", () => {
      instance.size.should.be.eql(7);
    });
  });
  describe("#parse", () => {
    it("should return somethind that is not undefined", () => {
      terminatorHeader.should.not.be.undefined;
    });
    it("should have a crc value as 2 bytes", () => {
      terminatorHeader.crc.should.not.be.undefiend;
      terminatorHeader.crc.should.be.eql(0x3DC4);
      new TerminatorHeaderParser(mockStreamFromString("4444")).parse().crc.should.be.eql(0x4444);
      new TerminatorHeaderParser(mockStreamFromString("1234")).parse().crc.should.be.eql(0x3412);
      new TerminatorHeaderParser(mockStreamFromString("ABBA")).parse().crc.should.be.eql(0xBAAB);
      new TerminatorHeaderParser(mockStreamFromString("0000")).parse().crc.should.be.eql(0x0000);
    });
    it("should parse crc in little endian format", () => {
      new TerminatorHeaderParser(mockStreamFromString("3412")).parse().crc.should.be.eql(0x1234);
      new TerminatorHeaderParser(mockStreamFromString("1234")).parse().crc.should.be.eql(0x3412);
    });
    it("should have a type value as 1 byte", () => {
      terminatorHeader.type.should.not.be.undefiend;
      terminatorHeader.type.should.be.eql(0x7B);
      new TerminatorHeaderParser(mockStreamFromString("000074")).parse().type.should.be.eql(0x74);
      new TerminatorHeaderParser(mockStreamFromString("000045")).parse().type.should.be.eql(0x45);
      new TerminatorHeaderParser(mockStreamFromString("000055")).parse().type.should.be.eql(0x55);
      new TerminatorHeaderParser(mockStreamFromString("123474")).parse().type.should.be.eql(0x74);
      new TerminatorHeaderParser(mockStreamFromString("FFFF74")).parse().type.should.be.eql(0x74);
    });
    it("should parse flags as 2 bytes", () => {
      terminatorHeader.flags.should.not.be.undefiend;
      terminatorHeader.flags.should.be.eql(0x4000);
      let parser = new TerminatorHeaderParser(mockStreamFromString("0000004444"));
      parser.parse().flags.should.be.eql(0x4444);
      parser = new TerminatorHeaderParser(mockStreamFromString("0000001234")).parse();
      parser.flags.should.be.eql(0x3412);
      parser = new TerminatorHeaderParser(mockStreamFromString("000000ABBA")).parse();
      parser.flags.should.be.eql(0xBAAB);
      parser = new TerminatorHeaderParser(mockStreamFromString("FFFFFF0000")).parse();
      parser.flags.should.be.eql(0x0000);
      parser = new TerminatorHeaderParser(mockStreamFromString("0000000000")).parse();
      parser.flags.should.be.eql(0x0000);
    });
    it("should parse flags as little endian", () => {
      let parser = new TerminatorHeaderParser(mockStreamFromString("123456789A")).parse();
      parser.flags.should.be.eql(0x9A78);
      parser = new TerminatorHeaderParser(mockStreamFromString("1234569A78")).parse();
      parser.flags.should.be.eql(0x789A);
    });
    it("should parse size as 2 bytes", () => {
      terminatorHeader.size.should.not.be.undefiend;
      terminatorHeader.size.should.be.eql(0x7);
      let parser = new TerminatorHeaderParser(mockStreamFromString("123456789A0700")).parse();
      parser.size.should.be.eql(0x7);
      parser = new TerminatorHeaderParser(mockStreamFromString("123456789A1111")).parse();
      parser.size.should.be.eql(0x1111);
    });
    it("should parse size as 2 bytes as little endian", () => {
      let parser = new TerminatorHeaderParser(mockStreamFromString("00000000004321")).parse();
      parser.size.should.be.eql(0x2143);
      parser = new TerminatorHeaderParser(mockStreamFromString("00000000001234")).parse();
      parser.size.should.be.eql(0x3412);
    });
  });
});
