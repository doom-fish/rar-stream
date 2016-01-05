import chai from "chai";
import sinon from "sinon";
import sinonChai from "sinon-chai";
let expect = chai.expect;
chai.should();
chai.use(sinonChai);

import AbstractFileMedia from "../../src/file/abstract-file-media";
import FileMedia from "../../src/file/file-media";

describe("FileMedia", () => {
  let instance;
  beforeEach(() => {
    instance = new FileMedia({
      name: "Instance",
      createReadStream: () => ({on: (name, cb) => cb()})
    });
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      expect(instance).to.be.an.instanceOf(FileMedia);
    });
    it("should inherit from FileMedia", () => {
      expect(instance).to.be.an.instanceOf(AbstractFileMedia);
    });
    it("should throw if options are empty", () => {
      expect(() => new FileMedia()).to.throw(/Invalid Arguments/);
    });
  });
  describe("#name", () => {
    it("should take a name in constructor and expose through name getter", () => {
      let namedTorrentMedia = new FileMedia({name: "Named Media"});
      expect(namedTorrentMedia.name).to.equal("Named Media");
    });
  });
  describe("#size", () => {
    it("should take a length in constructor and expose through size getter", () => {
      let sizedTorrentMedia = new FileMedia({size: 1337});
      expect(sizedTorrentMedia.size).to.equal(1337);

      sizedTorrentMedia = new FileMedia({size: 5201});
      expect(sizedTorrentMedia.size).to.equal(5201);
    });
  });
  describe("#createReadStream", () => {
    it("should throw if start is greater than end", () => {
      expect(() => instance.createReadStream(2, 0)).to.throw(/Invalid Arguments/);
    });
    it("should return a promise", () => {
      expect(instance.createReadStream(0, 0)).to.be.an.instanceOf(Promise);
    });
    it("should return a readable stream", () => {
      return instance.createReadStream(0, 0).should.eventually.be.fulfilled;
    });
    it("should call torrent createReadStream with offset arguments", () => {
      let torrentFile = {createReadStream: sinon.spy()};
      torrentFile.on = sinon.spy();

      let torrentCreateReadStreamInstance = new FileMedia(torrentFile);
      torrentCreateReadStreamInstance.createReadStream(0, 20);
      torrentFile.createReadStream.should.have.been.calledWith(0, 20);
    });
    it("should subscribe to events", () => {
      let spy = sinon.spy();
      let torrentFile = {
        createReadStream: () => ({
          on: (name, cb) => {
            spy(name);
            cb();
          }
        })
      };

      let torrentCreateReadStreamInstance = new FileMedia(torrentFile);
      torrentCreateReadStreamInstance.createReadStream(0, 20).then((done) => {
        spy.should.have.been.calledWith("readable");
        spy.should.have.been.calledWith("error");
        done();
      });
    });
  });
});
