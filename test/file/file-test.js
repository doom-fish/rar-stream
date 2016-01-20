import chai from "chai";
let expect = chai.expect;

import File from "../../src/file/file";

describe("File", () => {
  let file;
  beforeEach(() => {
    file = new File({name: "", size: 0});
  });
  describe("#constructor", () => {
    it("should be constructable", () => {
      file.should.be.an.instanceOf(File);
    });
    it("should take file options as parameter", () => {
      expect(() => new File()).to.throw(/Invalid Arguments/);
    });
  });
  describe("#size", () => {
    it("it should be defined", () => {
      file.size.should.not.be.undefined;
      new File({name: "", size: 10}).size.should.be.eql(10);
      new File({name: "", size: 5321}).size.should.be.eql(5321);
      new File({name: "", size: 59603102412}).size.should.be.eql(59603102412);
    });
    it("should not be able to be negative", () => {
      expect(() => new File({name: "", size: -12351235})).to.throw(/Invalid Arguments/);
      expect(() => new File({name: "", size: -3529195123})).to.throw(/Invalid Arguments/);
      expect(() => new File({name: "", size: -Infinity})).to.throw(/Invalid Arguments/);
    });
  });
  describe("#name", () => {
    it("should be defined", () => {
      file.name.should.not.be.undefined;
    });
    it("should throw if not sent to constructor", () => {
      expect(() => new File({size: 0})).to.throw(/Invalid Arguments/);
    });
    it("should throw if not a string", () => {
      expect(() => new File({size: 0, name: 324234})).to.throw(/Invalid Arguments/);
      expect(() => new File({size: 0, name: NaN})).to.throw(/Invalid Arguments/);
      expect(() => new File({size: 0, name: null})).to.throw(/Invalid Arguments/);
      expect(() => new File({size: 0, name: {}})).to.throw(/Invalid Arguments/);
    });
    it("should be set reflecting what is set in constructor", () => {
      new File({size: 0, name: "hej"}).name.should.be.eql("hej");
      new File({size: 0, name: "test123121"}).name.should.be.eql("test123121");
      new File({size: 0, name: "awejf89329fj9283f"}).name.should.be.eql("awejf89329fj9283f");
      new File({size: 0, name: "1234567876543"}).name.should.be.eql("1234567876543");
      new File({size: 0, name: ""}).name.should.be.eql("");
    })
  });

});
