import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {mockStreamFromString} from "../mocks/mock-buffer-stream";
chai.use(chaiAsPromised);
let expect = chai.expect;

import AbstractParser from "../../src/parsing/abstract-parser";
import MockAbstractParser from "../mocks/mock-abstract-parser";

describe("AbstractParserTest", () => {
  let instance;
  beforeEach(() => {
    instance = new AbstractParser(mockStreamFromString("00"));
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      expect(instance).to.be.an.instanceof(AbstractParser);
    });
  });
  describe("#size", () => {
    it("should throw as getter is abstract", () => {
      expect(() => instance.size).to.throw(/Abstract Getter/);
    });
  });
  describe("#parse", () => {
    it("should throw as method is abstract", () => {
      expect(() => instance.parse()).to.throw(/Abstract Method/);
    });
  });
  describe("#read", () => {
    it("should read from stream and return a buffer", () => {
      let withSizeInstance = new MockAbstractParser(mockStreamFromString("AF"), 1);
      let withSizeInstanceResult = withSizeInstance.read(1);
      withSizeInstanceResult.length.should.be.eql(1);
      withSizeInstanceResult.should.be.eql(new Buffer("AF", "hex"));

      let stream = mockStreamFromString("0123456789ABCDEF", { size: 8 });
      let withBiggerBuffer = new MockAbstractParser(stream, 8);
      let withBiggerBufferResult = withBiggerBuffer.read(8);
      withBiggerBufferResult.length.should.be.eql(8);
      withBiggerBufferResult.should.be.eql(new Buffer("0123456789ABCDEF", "hex"));
    });
  });
});
