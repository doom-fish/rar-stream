import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {mockStreamFromString} from "../mocks/mock-buffer-stream";
chai.use(chaiAsPromised);
let expect = chai.expect;


import AbstractParser from "../../src/parsing/abstract-parser";

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
});
