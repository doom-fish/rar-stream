import chai from "chai";
let expect = chai.expect;

import {MockEmptyParser} from "../mocks/mock-parsers";

import MockRarFile from "../../src/file/rar-file";
import AbstractFileMedia from "../../src/file/abstract-file-media";


MockRarFile.__Rewire__({
  MarkerHeaderParser: MockEmptyParser,
  ArchiveHeaderParser: MockEmptyParser,
  FileHeaderParser: class MockFileHeaderParser {
    parse() {
      return [
        "test"
      ];
    }
  },
  TerminatorHeaderParser: MockEmptyParser
});

describe("RarFile", () => {
  describe("#constructor", () => {
    // it("should be constructable", () => {
    //   let rarFile = new MockRarFile(new AbstractFileMedia());
    //   rarFile.should.be.an.instanceOf(MockRarFile);
    // });
    // it("should take a file medium as parameter", () => {
    //   expect(() => new MockRarFile()).to.throw(/Invalid Arguments/);
    // });
  });
  describe("#files", () => {
    // it("should give back a list of files based on the file medium", () => {
    //   let rarFile = new MockRarFile(new AbstractFileMedia());
    //   rarFile.files.should.be.eql(["test"]);
    // });
  });
});
