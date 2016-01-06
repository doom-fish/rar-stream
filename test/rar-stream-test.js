import {Readable} from "stream";
import chai from "chai";
let expect = chai.expect;

chai.should();

import RarFileBundle from "../src/rar-file-bundle";
import RarStream from "../src/rar-stream";


describe("RarStream", () => {
  let simpleRarFileBundle;
  beforeEach(() => {
    simpleRarFileBundle = new RarFileBundle(["1.rar", "1.r00"]);
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      expect(new RarStream(simpleRarFileBundle)).to.be.an.instanceOf(RarStream);
      expect(new RarStream(simpleRarFileBundle)).to.be.an.instanceOf(Readable);
    });
    it("should throw with empty fileBundle argument", () => {
      expect(() => new RarStream()).to.throw(/Invalid Arguments/);
    });
  });
});
