import chai from "chai";
import chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);
let expect = chai.expect;


import AbstractParser from "../../src/parsing/abstract-parser";

describe("AbstractParserTest", () => {
  let instance;
  beforeEach(() => {
    instance = new AbstractParser(new Buffer(0));
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      expect(instance).to.be.an.instanceof(AbstractParser);
    });
  });
  describe("#parse", () => {
    it("should throw as method is abstract", () => {
      expect(() => instance.parse()).to.throw(/Abstract Method/);
    });
  });
});
