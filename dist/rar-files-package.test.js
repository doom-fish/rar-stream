//
import { expect, test } from "vitest";
import path from "path";
import fs from "fs";
import { RarFilesPackage } from "./rar-files-package.js";
import { streamToBuffer } from "./stream-utils.js";
import { makeRarFileBundle } from "./rar-file-bundle.js";
import { LocalFileMedia } from "./local-file-media.js";
const fixturePath = path.resolve(__dirname, "./__fixtures__");
const singleFilePath = path.resolve(fixturePath, "single/single.txt");
const multiFilePath = path.resolve(fixturePath, "multi/multi.txt");
const singleSplitted1FilePath = path.resolve(fixturePath, "single-splitted/splitted1.txt");
const singleSplitted2FilePath = path.resolve(fixturePath, "single-splitted/splitted2.txt");
const singleSplitted3FilePath = path.resolve(fixturePath, "single-splitted/splitted3.txt");
const multiSplitted1FilePath = path.resolve(fixturePath, "multi-splitted/splitted1.txt");
const multiSplitted2FilePath = path.resolve(fixturePath, "multi-splitted/splitted2.txt");
const multiSplitted3FilePath = path.resolve(fixturePath, "multi-splitted/splitted3.txt");
const multiSplitted4FilePath = path.resolve(fixturePath, "multi-splitted/splitted4.txt");
const singleFileRarWithOneInnerFile = [
    path.resolve(fixturePath, "single/single.rar"),
].map((a) => new LocalFileMedia(a));
const singleRarWithManyInnerFiles = [
    path.resolve(fixturePath, "single-splitted/single-splitted.rar"),
].map((a) => new LocalFileMedia(a));
const multipleRarFileWithOneInnerFile = [
    path.resolve(fixturePath, "multi/multi.rar"),
    path.resolve(fixturePath, "multi/multi.r01"),
    path.resolve(fixturePath, "multi/multi.r00"),
].map((a) => new LocalFileMedia(a));
const multipleRarFileWithManyInnerFiles = [
    path.resolve(fixturePath, "multi-splitted/multi-splitted.rar"),
    path.resolve(fixturePath, "multi-splitted/multi-splitted.r00"),
    path.resolve(fixturePath, "multi-splitted/multi-splitted.r01"),
].map((a) => new LocalFileMedia(a));
const readToEnd = (f) => Promise.all(f.map((file) => file.readToEnd()));
test("rar package emits events for when parsing ends", async () => {
    const rarPackage = new RarFilesPackage(multipleRarFileWithOneInnerFile);
    let eventResult;
    rarPackage.on("parsing-complete", (files) => {
        eventResult = files;
    });
    const files = await rarPackage.parse();
    expect(eventResult).toBe(files);
});
test("rar package emits events for when parsing starts", async () => {
    const rarPackage = new RarFilesPackage(multipleRarFileWithOneInnerFile);
    const bundle = makeRarFileBundle(multipleRarFileWithOneInnerFile);
    rarPackage.on("parsing-start", () => expect(bundle.fileNames).toEqual(bundle.fileNames));
    await rarPackage.parse();
});
test("rar package emits events for each parsed file", async () => {
    const bundle = makeRarFileBundle(multipleRarFileWithOneInnerFile);
    const rarPackage = new RarFilesPackage(multipleRarFileWithOneInnerFile);
    let i = 0;
    rarPackage.on("file-parsed", (file) => {
        expect(file.name).toBe(bundle.fileNames[i++]);
    });
    await rarPackage.parse();
});
test("single rar file with one inner file can be read as whole", async () => {
    const rarPackage = new RarFilesPackage(singleFileRarWithOneInnerFile);
    const files = await rarPackage.parse();
    const [rarFileContent] = await readToEnd(files);
    const singleFileContent = fs.readFileSync(singleFilePath);
    expect(rarFileContent?.length).toBe(singleFileContent.length);
    expect(rarFileContent).toEqual(singleFileContent);
});
test("single rar file with one inner files can be read in parts", async () => {
    const interval = { start: 53, end: 1000 };
    const rarPackage = new RarFilesPackage(singleFileRarWithOneInnerFile);
    const [file] = await rarPackage.parse();
    const rarFileInterval = file?.createReadStream(interval);
    const singleFileInterval = fs.createReadStream(singleFilePath, interval);
    const rarFileBuffer = await streamToBuffer(rarFileInterval);
    const singleFileBuffer = await streamToBuffer(singleFileInterval);
    expect(rarFileBuffer.length).toBe(singleFileBuffer.length);
    expect(rarFileBuffer).toEqual(singleFileBuffer);
});
test("single rar file with many inner files can be read as whole", async () => {
    const rarPackage = new RarFilesPackage(singleRarWithManyInnerFiles);
    const [rarFile1, rarFile2, rarFile3] = await rarPackage
        .parse()
        .then(readToEnd);
    const splitted1 = fs.readFileSync(singleSplitted1FilePath);
    const splitted2 = fs.readFileSync(singleSplitted2FilePath);
    const splitted3 = fs.readFileSync(singleSplitted3FilePath);
    expect(rarFile1.length).toBe(splitted1.length);
    expect(rarFile2.length).toBe(splitted2.length);
    expect(rarFile3.length).toBe(splitted3.length);
    expect(rarFile1).toEqual(splitted1);
    expect(rarFile2).toEqual(splitted2);
    expect(rarFile3).toEqual(splitted3);
});
test("single rar file with many inner files can be read in parts", async () => {
    const interval = { start: 50, end: 200 };
    const rarPackage = new RarFilesPackage(singleRarWithManyInnerFiles);
    const [rarFile1, rarFile2, rarFile3] = await rarPackage.parse();
    const rarFile1Buffer = await streamToBuffer(rarFile1.createReadStream(interval));
    const rarFile2Buffer = await streamToBuffer(rarFile2.createReadStream(interval));
    const rarFile3Buffer = await streamToBuffer(rarFile3.createReadStream(interval));
    const splittedFile1Buffer = await streamToBuffer(fs.createReadStream(singleSplitted1FilePath, interval));
    const splittedFile2Buffer = await streamToBuffer(fs.createReadStream(singleSplitted2FilePath, interval));
    const splittedFile3Buffer = await streamToBuffer(fs.createReadStream(singleSplitted3FilePath, interval));
    expect(rarFile1Buffer.length).toBe(splittedFile1Buffer.length);
    expect(rarFile2Buffer.length).toBe(splittedFile2Buffer.length);
    expect(rarFile3Buffer.length).toBe(splittedFile3Buffer.length);
    expect(rarFile1Buffer).toEqual(splittedFile1Buffer);
    expect(rarFile2Buffer).toEqual(splittedFile2Buffer);
    expect(rarFile3Buffer).toEqual(splittedFile3Buffer);
});
//
test("multiple rar file with one inner can be read as a whole", async () => {
    const rarPackage = new RarFilesPackage(multipleRarFileWithOneInnerFile);
    const [rarFileBuffer] = await rarPackage.parse().then(readToEnd);
    const multiFile = fs.readFileSync(multiFilePath);
    expect(rarFileBuffer.length).toBe(multiFile.length);
    expect(rarFileBuffer).toEqual(multiFile);
});
test("multiple rar file with one inner can be read as in parts", async () => {
    const interval = { start: 0, end: 100 };
    const rarPackage = new RarFilesPackage(multipleRarFileWithOneInnerFile);
    const [file] = await rarPackage.parse();
    const rarFileBuffer = await streamToBuffer(file.createReadStream(interval));
    const multiFileBuffer = await streamToBuffer(fs.createReadStream(multiFilePath, interval));
    expect(rarFileBuffer.length).toBe(multiFileBuffer.length);
    expect(rarFileBuffer).toEqual(multiFileBuffer);
});
test("multi rar file with many inner files can be read as whole", async () => {
    const rarPackage = new RarFilesPackage(multipleRarFileWithManyInnerFiles);
    const [rarFile1, rarFile2, rarFile3, rarFile4] = await rarPackage
        .parse()
        .then(readToEnd);
    const splitted1 = fs.readFileSync(multiSplitted1FilePath);
    const splitted2 = fs.readFileSync(multiSplitted2FilePath);
    const splitted3 = fs.readFileSync(multiSplitted3FilePath);
    const splitted4 = fs.readFileSync(multiSplitted4FilePath);
    expect(rarFile1.length).toBe(splitted1.length);
    expect(rarFile2.length).toBe(splitted2.length);
    expect(rarFile3.length).toBe(splitted3.length);
    expect(rarFile4.length).toBe(splitted4.length);
});
test("multi rar file with many inner files can be read in parts", async () => {
    const interval = { start: 56, end: 200 };
    const rarPackage = new RarFilesPackage(multipleRarFileWithManyInnerFiles);
    const [rarFile1, rarFile2, rarFile3, rarFile4] = await rarPackage.parse();
    const rarFile1Buffer = await streamToBuffer(rarFile1.createReadStream(interval));
    const rarFile2Buffer = await streamToBuffer(rarFile2.createReadStream(interval));
    const rarFile3Buffer = await streamToBuffer(rarFile3.createReadStream(interval));
    const rarFile4Buffer = await streamToBuffer(rarFile4.createReadStream(interval));
    const splittedFile1Buffer = await streamToBuffer(fs.createReadStream(multiSplitted1FilePath, interval));
    const splittedFile2Buffer = await streamToBuffer(fs.createReadStream(multiSplitted2FilePath, interval));
    const splittedFile3Buffer = await streamToBuffer(fs.createReadStream(multiSplitted3FilePath, interval));
    const splittedFile4Buffer = await streamToBuffer(fs.createReadStream(multiSplitted4FilePath, interval));
    expect(rarFile1Buffer.length).toBe(splittedFile1Buffer.length);
    expect(rarFile2Buffer.length).toBe(splittedFile2Buffer.length);
    expect(rarFile3Buffer.length).toBe(splittedFile3Buffer.length);
    expect(rarFile4Buffer.length).toBe(splittedFile4Buffer.length);
    expect(rarFile1Buffer).toEqual(splittedFile1Buffer);
    expect(rarFile2Buffer).toEqual(splittedFile2Buffer);
    expect(rarFile3Buffer).toEqual(splittedFile3Buffer);
    expect(rarFile4Buffer).toEqual(splittedFile4Buffer);
});
