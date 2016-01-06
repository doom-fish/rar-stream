import chai from "chai";
import chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);
let expect = chai.expect;

import TerminatorHeaderParser from "../../src/parsing/terminator-header-parser";
import AbstractParser from "../../src/parsing/abstract-parser.js";

let terminatorHeaderBuffer = new Buffer(0);

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
      expect(terminatorHeader).to.not.be.undefined;
    });
  });
});
