import chai from "chai";
let expect = chai.expect;
import {Readable} from "stream";
import File from "../../src/file/file";

describe("File", () => {
  let file;
  beforeEach(() => {
    file = new File({name: "", size: 0, stream: new Readable()});
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
      new File({stream: new Readable(), name: "", size: 10}).size.should.be.eql(10);
      new File({stream: new Readable(), name: "", size: 5321}).size.should.be.eql(5321);
      new File({stream: new Readable(), name: "", size: 596031024}).size.should.be.eql(596031024);
    });
    it("should not be able to be negative", () => {
      expect(() => new File({
        stream: new Readable(),
        name: "",
        size: -12351235})).to.throw(/Invalid Arguments/);
      expect(() => new File({
        stream: new Readable(),
        name: "",
        size: -3529195123})).to.throw(/Invalid Arguments/);
      expect(() => new File({
        stream: new Readable(),
        name: "",
        size: -Infinity})).to.throw(/Invalid Arguments/);
    });
  });
  describe("#name", () => {
    it("should be defined", () => {
      file.name.should.not.be.undefined;
    });
    it("should throw if not sent to constructor", () => {
      expect(() => new File({stream: new Readable(), size: 0})).to.throw(/Invalid Arguments/);
    });
    it("should throw if not a string", () => {
      expect(() => new File({
        stream: new Readable(),
        size: 0,
        name: 324234})).to.throw(/Invalid Arguments/);
      expect(() => new File({
        stream: new Readable(),
        size: 0,
        name: NaN})).to.throw(/Invalid Arguments/);
      expect(() => new File({
        stream: new Readable(),
        size: 0,
        name: null})).to.throw(/Invalid Arguments/);
      expect(() => new File({
        stream: new Readable(),
        size: 0,
        name: {}})).to.throw(/Invalid Arguments/);
    });
    it("should be set reflecting what is set in constructor", () => {
      new File({
        stream: new Readable(),
        size: 0,
        name: "hej"}).name.should.be.eql("hej");
      new File({
        stream: new Readable(),
        size: 0,
        name: "test123121"}).name.should.be.eql("test123121");
      new File({
        stream: new Readable(),
        size: 0,
        name: "awejf89329fj9283f"}).name.should.be.eql("awejf89329fj9283f");
      new File({
        stream: new Readable(),
        size: 0,
        name: "1234567876543"}).name.should.be.eql("1234567876543");
      new File({
        stream: new Readable(),
        size: 0,
        name: ""}).name.should.be.eql("");
    });
    describe("#stream", () => {
      it("should be defined", () => {
        file.stream.should.not.be.undefined;
      });
      it("should be of a ReadableStream", () => {
        file.stream.should.be.an.instanceof(Readable);
      });
      it("should be same stream as passed through constructor", () => {
        let readable = new Readable();
        new File({stream: readable, size: 0, name: ""}).stream.should.be.equal(readable);
      });
      it("should throw if not passed as a ReadableStream to constructor", () => {
        expect(() => new File({name: "", size: 0})).to.throw(/Invalid Arguments/);
      });
    });
  });

});
