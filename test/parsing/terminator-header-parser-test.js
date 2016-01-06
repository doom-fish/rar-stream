import chai from "chai";
import chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);

import TerminatorHeaderParser from "../../src/parsing/terminator-header-parser";
import AbstractParser from "../../src/parsing/abstract-parser.js";

let terminatorHeaderBuffer = new Buffer("C43D7B00400700", "hex");

describe("TerminatorHeaderParserTest", () => {
  let instance;
  let terminatorHeader;
  beforeEach(() => {
    instance = new TerminatorHeaderParser(terminatorHeaderBuffer);
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
  describe("#parse", () => {
    it("should return somethind that is not undefined", () => {
      terminatorHeader.should.not.be.undefined;
    });
    it("should have a crc value as 2 bytes", () => {
      terminatorHeader.crc.should.not.be.undefiend;
      terminatorHeader.crc.should.be.eql(0x3DC4);
      new TerminatorHeaderParser(new Buffer("4444", "hex")).parse().crc.should.be.eql(0x4444);
      new TerminatorHeaderParser(new Buffer("1234", "hex")).parse().crc.should.be.eql(0x3412);
      new TerminatorHeaderParser(new Buffer("ABBA", "hex")).parse().crc.should.be.eql(0xBAAB);
      new TerminatorHeaderParser(new Buffer("0000", "hex")).parse().crc.should.be.eql(0x0000);
    });
    it("should parse crc in little endian format", () => {
      new TerminatorHeaderParser(new Buffer("3412", "hex")).parse().crc.should.be.eql(0x1234);
      new TerminatorHeaderParser(new Buffer("1234", "hex")).parse().crc.should.be.eql(0x3412);
    });
    it("should have a type value as 1 byte", () => {
      terminatorHeader.type.should.not.be.undefiend;
      terminatorHeader.type.should.be.eql(0x7B);
      new TerminatorHeaderParser(new Buffer("000074", "hex")).parse().type.should.be.eql(0x74);
      new TerminatorHeaderParser(new Buffer("000045", "hex")).parse().type.should.be.eql(0x45);
      new TerminatorHeaderParser(new Buffer("000055", "hex")).parse().type.should.be.eql(0x55);
      new TerminatorHeaderParser(new Buffer("123474", "hex")).parse().type.should.be.eql(0x74);
      new TerminatorHeaderParser(new Buffer("FFFF74", "hex")).parse().type.should.be.eql(0x74);
    });
    it("should parse flags as 2 bytes", () => {
      terminatorHeader.flags.should.not.be.undefiend;
      terminatorHeader.flags.should.be.eql(0x4000);
      let parser = new TerminatorHeaderParser(new Buffer("0000004444", "hex"));
      parser.parse().flags.should.be.eql(0x4444);
      parser = new TerminatorHeaderParser(new Buffer("0000001234", "hex")).parse();
      parser.flags.should.be.eql(0x3412);
      parser = new TerminatorHeaderParser(new Buffer("000000ABBA", "hex")).parse();
      parser.flags.should.be.eql(0xBAAB);
      parser = new TerminatorHeaderParser(new Buffer("FFFFFF0000", "hex")).parse();
      parser.flags.should.be.eql(0x0000);
      parser = new TerminatorHeaderParser(new Buffer("0000000000", "hex")).parse();
      parser.flags.should.be.eql(0x0000);
    });
    it("should parse flags as little endian", () => {
      let parser = new TerminatorHeaderParser(new Buffer("123456789A", "hex")).parse();
      parser.flags.should.be.eql(0x9A78);
      parser = new TerminatorHeaderParser(new Buffer("1234569A78", "hex")).parse();
      parser.flags.should.be.eql(0x789A);
    });
    it("should parse size as 2 bytes", () => {
      terminatorHeader.size.should.not.be.undefiend;
      terminatorHeader.size.should.be.eql(0x7);
      let parser = new TerminatorHeaderParser(new Buffer("123456789A0700", "hex")).parse();
      parser.size.should.be.eql(0x7);
      parser = new TerminatorHeaderParser(new Buffer("123456789A1111", "hex")).parse();
      parser.size.should.be.eql(0x1111);
    });
    it("should parse size as 2 bytes as little endian", () => {
      let parser = new TerminatorHeaderParser(new Buffer("00000000004321", "hex")).parse();
      parser.size.should.be.eql(0x2143);
      parser = new TerminatorHeaderParser(new Buffer("00000000001234", "hex")).parse();
      parser.size.should.be.eql(0x3412);
    });
  });
});
