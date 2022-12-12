import { expect, test } from "vitest";
import { IFileMedia } from "./interfaces.js";
import { Readable } from "stream";
import { makeRarFileBundle } from "./rar-file-bundle.js";

const newFileMedia = (name: string) =>
  ({
    name,
    length: 0,
    createReadStream: () => new Readable(),
  } satisfies IFileMedia);

test("RarFileBundle length should be 0 with an empty array as input", () => {
  const emptyInstance = makeRarFileBundle();
  expect(emptyInstance.length).toBe(0);
});

test("RarFileBundle should return length with the same length as input", () => {
  const input = ["a.r01", "a.r02", "a.r03", "a.r04", "a.r05"];
  const fileMedias = input.map(newFileMedia);
  const inputInstance = makeRarFileBundle(fileMedias);
  expect(inputInstance.length).toBe(input.length);
});

test("RarFileBundle should deconstruct into input", () => {
  const input = ["a.r01", "a.r02", "a.r03", "a.r04", "a.r05"];
  const fileMedias = input.map(newFileMedia);
  const inputInstance = makeRarFileBundle(fileMedias);
  expect(fileMedias).toEqual(inputInstance.files);
});

test("RarFileBundle should return unsorted rxx filenames in a sorted manner", () => {
  const unsortedFileNames = ["a.r03", "a.r02", "a.rar", "a.r01", "a.r00"];
  const fileMedias = unsortedFileNames.map(newFileMedia);
  const sortedFileNames = ["a.rar", "a.r00", "a.r01", "a.r02", "a.r03"];
  const instanceWithUnsortedParameters = makeRarFileBundle(fileMedias);

  expect(instanceWithUnsortedParameters.fileNames).toEqual(sortedFileNames);
});

test("RarFileBundle should return unsorted part file names in a sorted manner", () => {
  const sortedFileNames = [
    "a.part01.rar",
    "a.part02.rar",
    "a.part03.rar",
    "a.part04.rar",
    "a.part05.rar",
    "a.part06.rar",
  ];

  const unsortedFileNames = [
    "a.part06.rar",
    "a.part01.rar",
    "a.part04.rar",
    "a.part03.rar",
    "a.part05.rar",
    "a.part02.rar",
  ];
  const fileMedias = unsortedFileNames.map(newFileMedia);

  const instanceWithUnsortedParameters = makeRarFileBundle(fileMedias);
  expect(instanceWithUnsortedParameters.fileNames).toEqual(sortedFileNames);
});

test("RarFileBundle should filter out non rar files", () => {
  const unfilteredFileNames = [
    "a.part01.rar",
    "a.part02.rar",
    "a.part03.rar",
    "a.sfv",
    "a.jpg",
    "a.part04.rar",
    "a.nfo",
    "a.part05.rar",
  ];
  const fileMedias = unfilteredFileNames.map(newFileMedia);

  const filteredFileNames = [
    "a.part01.rar",
    "a.part02.rar",
    "a.part03.rar",
    "a.part04.rar",
    "a.part05.rar",
  ];
  const unFilteredInstance = makeRarFileBundle(fileMedias);
  expect(unFilteredInstance.fileNames).toEqual(filteredFileNames);
});
