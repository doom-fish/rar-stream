import chai from "chai";
let expect = chai.expect;

import MockFileMedia from "../mocks/mock-file-media";
import RarFile from "../../src/file/rar-file";

describe("RarFile", () => {
  let rarFile;
  beforeEach(() => {
    rarFile = new RarFile(new MockFileMedia("00"));
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      rarFile.should.be.an.instanceOf(RarFile);
    });
    it("should take a file medium as parameter", () => {
      expect(() => new RarFile()).to.throw(/Invalid Arguments/);
    });
  });
  describe("#parse", () => {
    it("should be accessible after instance is created", () => {

    });
  });

});
