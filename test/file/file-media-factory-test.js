import chai from "chai";
import path from "path";

let expect = chai.expect;
chai.should();

import FileMediaFactory from "../../src/file/file-media-factory";
import FileMediaTypes from "../../src/file/file-media-types";
import TorrentFileMedia from "../../src/file/torrent-file-media";
import LocalFileMedia from "../../src/file/local-file-media";

describe("FileMediaFactory", () => {
  describe("#resolve", () => {
    it("should throw if called without arguments", () => {
      expect(() => FileMediaFactory.createInstance()).to.throw(/Invalid Arguments/);
    });
    it("should return a TorrentFileMedia if file type torrent is passed", () => {
      let instance = FileMediaFactory.createInstance(FileMediaTypes.TORRENT, {});
      expect(instance).to.be.an.instanceOf(TorrentFileMedia);
    });
    it("should return a LocalFileMedia if file type local is passed", () => {
      let localFilePath = path.resolve(__dirname, "./file-media-factory-test.js");
      let instance = FileMediaFactory.createInstance(FileMediaTypes.LOCAL, localFilePath);
      expect(instance).to.be.an.instanceOf(LocalFileMedia);
    });
  });
});
