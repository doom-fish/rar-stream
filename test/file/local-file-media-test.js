import chai from "chai";
import path from "path";
let expect = chai.expect;

let singleFile = path.resolve(__dirname, "../binary-data/single-short-filename.rar");
import LocalFileMedia from "../../src/file/local-file-media";

describe("LocalFileMedia", () => {
  describe("#constructor", () => {
    it("should be constructable", () => {
      let localFilePath = path.resolve(__dirname, "./local-file-media-test.js");
      expect(new LocalFileMedia(localFilePath)).to.be.an.instanceOf(LocalFileMedia);
    });
    it("should throw if constructor parameter is not a string", () => {
      expect(() => new LocalFileMedia(1)).to.throw(/Invalid Arguments/);
      expect(() => new LocalFileMedia()).to.throw(/Invalid Arguments/);
      expect(() => new LocalFileMedia({})).to.throw(/Invalid Arguments/);
      expect(() => new LocalFileMedia(null)).to.throw(/Invalid Arguments/);
    });
    it("should throw if path does not point to a local file", () => {
      let notFoundException = /ENOENT: no such file or directory/;
      expect(() => new LocalFileMedia("not a local file")).to.throw(notFoundException);
    });
    it("should parse path and read file size if string is passed as options", () => {
      let instance = new LocalFileMedia(singleFile);
      expect(instance.name).to.equal("single-short-filename.rar");
      expect(instance.size).to.equal(571016);
    });
  });
});
