'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _path = require('path');

var _path2 = _interopRequireDefault(_path);

var _fs = require('fs');

var _fs2 = _interopRequireDefault(_fs);

var _rarManifest = require('../rar-manifest');

var _rarManifest2 = _interopRequireDefault(_rarManifest);

var _rarFileBundle = require('../../rar-file/rar-file-bundle');

var _rarFileBundle2 = _interopRequireDefault(_rarFileBundle);

var _localFileMedia = require('../../file-media/local-file-media');

var _localFileMedia2 = _interopRequireDefault(_localFileMedia);

var _streamToBuffer = require('stream-to-buffer');

var _streamToBuffer2 = _interopRequireDefault(_streamToBuffer);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

function _asyncToGenerator(fn) { return function () { var gen = fn.apply(this, arguments); return new Promise(function (resolve, reject) { function step(key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { return Promise.resolve(value).then(function (value) { step("next", value); }, function (err) { step("throw", err); }); } } return step("next"); }); }; }

const streamToBufferPromise = (() => {
    var _ref = _asyncToGenerator(function* (stream) {
        return new Promise(function (resolve, reject) {
            return (0, _streamToBuffer2.default)(stream, function (err, buffer) {
                return err ? reject(err) : resolve(buffer);
            });
        });
    });

    return function streamToBufferPromise(_x) {
        return _ref.apply(this, arguments);
    };
})();

let fixturePath = _path2.default.join(__dirname, '../__fixtures__');
if (global.isBeingRunInWallaby) {
    fixturePath = global.fixturePath;
}

const singleFilePath = _path2.default.join(fixturePath, 'single/single.txt');
const multiFilePath = _path2.default.join(fixturePath, 'multi/multi.txt');

const singleSplitted1FilePath = _path2.default.join(fixturePath, 'single-splitted/splitted1.txt');
const singleSplitted2FilePath = _path2.default.join(fixturePath, 'single-splitted/splitted2.txt');
const singleSplitted3FilePath = _path2.default.join(fixturePath, 'single-splitted/splitted3.txt');

const multiSplitted1FilePath = _path2.default.join(fixturePath, 'multi-splitted/splitted1.txt');
const multiSplitted2FilePath = _path2.default.join(fixturePath, 'multi-splitted/splitted2.txt');
const multiSplitted3FilePath = _path2.default.join(fixturePath, 'multi-splitted/splitted3.txt');
const multiSplitted4FilePath = _path2.default.join(fixturePath, 'multi-splitted/splitted4.txt');

const createSingleFileRarBundle = () => new _rarFileBundle2.default(new _localFileMedia2.default(_path2.default.join(fixturePath, 'single/single.rar')));

const createSingleRarWithManyInnerBundle = () => new _rarFileBundle2.default(new _localFileMedia2.default(_path2.default.join(fixturePath, 'single-splitted/single-splitted.rar')));

const createMultipleRarFileWithOneInnerBundle = () => new _rarFileBundle2.default(new _localFileMedia2.default(_path2.default.join(fixturePath, 'multi/multi.rar')), new _localFileMedia2.default(_path2.default.join(fixturePath, 'multi/multi.r00')), new _localFileMedia2.default(_path2.default.join(fixturePath, 'multi/multi.r01')));

const createMultipleRarFileWithManyInnerBundle = () => new _rarFileBundle2.default(new _localFileMedia2.default(_path2.default.join(fixturePath, 'multi-splitted/multi-splitted.rar')), new _localFileMedia2.default(_path2.default.join(fixturePath, 'multi-splitted/multi-splitted.r00')), new _localFileMedia2.default(_path2.default.join(fixturePath, 'multi-splitted/multi-splitted.r01')));

const readToEnd = f => Promise.all(f.map(file => file.readToEnd()));

(0, _ava2.default)('single rar file with one inner files can be read as whole', (() => {
    var _ref2 = _asyncToGenerator(function* (t) {
        const bundle = createSingleFileRarBundle();
        const manifest = new _rarManifest2.default(bundle);
        const files = yield manifest.getFiles();
        const [rarFileContent] = yield readToEnd(files);
        const singleFileContent = _fs2.default.readFileSync(singleFilePath);
        t.is(rarFileContent.length, singleFileContent.length);
        t.deepEqual(rarFileContent, singleFileContent);
    });

    return function (_x2) {
        return _ref2.apply(this, arguments);
    };
})());

(0, _ava2.default)('single rar file with one inner files can be read in parts', (() => {
    var _ref3 = _asyncToGenerator(function* (t) {
        const bundle = createSingleFileRarBundle();
        const interval = { start: 50, end: 1000 };
        const manifest = new _rarManifest2.default(bundle);

        const [file] = yield manifest.getFiles();
        const rarFileInterval = file.createReadStream(interval);
        const singleFileInterval = _fs2.default.createReadStream(singleFilePath, interval);
        const rarFileBuffer = yield streamToBufferPromise(rarFileInterval);
        const singleFileBuffer = yield streamToBufferPromise(singleFileInterval);

        t.is(rarFileBuffer.length, singleFileBuffer.length);
        t.deepEqual(rarFileBuffer, singleFileBuffer);
    });

    return function (_x3) {
        return _ref3.apply(this, arguments);
    };
})());

(0, _ava2.default)('single rar file with many inner files can be read as whole', (() => {
    var _ref4 = _asyncToGenerator(function* (t) {
        const bundle = createSingleRarWithManyInnerBundle();
        const manifest = new _rarManifest2.default(bundle);
        const [rarFile1, rarFile2, rarFile3] = yield manifest.getFiles().then(readToEnd);

        const splitted1 = _fs2.default.readFileSync(singleSplitted1FilePath);
        const splitted2 = _fs2.default.readFileSync(singleSplitted2FilePath);
        const splitted3 = _fs2.default.readFileSync(singleSplitted3FilePath);

        t.is(rarFile1.length, splitted1.length);
        t.is(rarFile2.length, splitted2.length);
        t.is(rarFile3.length, splitted3.length);

        t.deepEqual(rarFile1, splitted1);
        t.deepEqual(rarFile2, splitted2);
        t.deepEqual(rarFile3, splitted3);
    });

    return function (_x4) {
        return _ref4.apply(this, arguments);
    };
})());

(0, _ava2.default)('single rar file with many inner files can be read in parts', (() => {
    var _ref5 = _asyncToGenerator(function* (t) {
        const bundle = createSingleRarWithManyInnerBundle();
        const interval = { start: 50, end: 200 };
        const manifest = new _rarManifest2.default(bundle);

        const [rarFile1, rarFile2, rarFile3] = yield manifest.getFiles();

        const rarFile1Buffer = yield streamToBufferPromise(rarFile1.createReadStream(interval));
        const rarFile2Buffer = yield streamToBufferPromise(rarFile2.createReadStream(interval));
        const rarFile3Buffer = yield streamToBufferPromise(rarFile3.createReadStream(interval));

        const splittedFile1Buffer = yield streamToBufferPromise(_fs2.default.createReadStream(singleSplitted1FilePath, interval));
        const splittedFile2Buffer = yield streamToBufferPromise(_fs2.default.createReadStream(singleSplitted2FilePath, interval));
        const splittedFile3Buffer = yield streamToBufferPromise(_fs2.default.createReadStream(singleSplitted3FilePath, interval));

        t.is(rarFile1Buffer.length, splittedFile1Buffer.length);
        t.is(rarFile2Buffer.length, splittedFile2Buffer.length);
        t.is(rarFile3Buffer.length, splittedFile3Buffer.length);

        t.deepEqual(rarFile1Buffer, splittedFile1Buffer);
        t.deepEqual(rarFile2Buffer, splittedFile2Buffer);
        t.deepEqual(rarFile3Buffer, splittedFile3Buffer);
    });

    return function (_x5) {
        return _ref5.apply(this, arguments);
    };
})());

(0, _ava2.default)('multiple rar file with one inner can be read as a whole', (() => {
    var _ref6 = _asyncToGenerator(function* (t) {
        const bundle = createMultipleRarFileWithOneInnerBundle();
        const manifest = new _rarManifest2.default(bundle);
        const [rarFileBuffer] = yield manifest.getFiles().then(readToEnd);
        const multiFile = _fs2.default.readFileSync(multiFilePath);
        t.is(rarFileBuffer.length, multiFile.length);
        t.deepEqual(rarFileBuffer.toString(), multiFile.toString());
    });

    return function (_x6) {
        return _ref6.apply(this, arguments);
    };
})());

(0, _ava2.default)('multiple rar file with one inner can be read as in parts', (() => {
    var _ref7 = _asyncToGenerator(function* (t) {
        const bundle = createMultipleRarFileWithOneInnerBundle();
        const interval = { start: 50, end: 100 };
        const manifest = new _rarManifest2.default(bundle);

        const [file] = yield manifest.getFiles();
        const rarFileBuffer = yield streamToBufferPromise(file.createReadStream(interval));
        const multiFileBuffer = yield streamToBufferPromise(_fs2.default.createReadStream(multiFilePath, interval));

        t.is(rarFileBuffer.length, multiFileBuffer.length);
        t.deepEqual(rarFileBuffer, multiFileBuffer);
    });

    return function (_x7) {
        return _ref7.apply(this, arguments);
    };
})());

(0, _ava2.default)('multi rar file with many inner files can be read as whole', (() => {
    var _ref8 = _asyncToGenerator(function* (t) {
        const bundle = createMultipleRarFileWithManyInnerBundle();
        const manifest = new _rarManifest2.default(bundle);
        const [rarFile1, rarFile2, rarFile3, rarFile4] = yield manifest.getFiles().then(readToEnd);

        const splitted1 = _fs2.default.readFileSync(multiSplitted1FilePath);
        const splitted2 = _fs2.default.readFileSync(multiSplitted2FilePath);
        const splitted3 = _fs2.default.readFileSync(multiSplitted3FilePath);
        const splitted4 = _fs2.default.readFileSync(multiSplitted4FilePath);

        t.is(rarFile1.length, splitted1.length);
        t.is(rarFile2.length, splitted2.length);
        t.is(rarFile3.length, splitted3.length);
        t.is(rarFile4.length, splitted4.length);
    });

    return function (_x8) {
        return _ref8.apply(this, arguments);
    };
})());

(0, _ava2.default)('multi rar file with many inner files can be read in parts', (() => {
    var _ref9 = _asyncToGenerator(function* (t) {
        const bundle = createMultipleRarFileWithManyInnerBundle();
        const interval = { start: 50, end: 200 };
        const manifest = new _rarManifest2.default(bundle);

        const [rarFile1, rarFile2, rarFile3, rarFile4] = yield manifest.getFiles();

        const rarFile1Buffer = yield streamToBufferPromise(rarFile1.createReadStream(interval));
        const rarFile2Buffer = yield streamToBufferPromise(rarFile2.createReadStream(interval));
        const rarFile3Buffer = yield streamToBufferPromise(rarFile3.createReadStream(interval));
        const rarFile4Buffer = yield streamToBufferPromise(rarFile4.createReadStream(interval));

        const splittedFile1Buffer = yield streamToBufferPromise(_fs2.default.createReadStream(multiSplitted1FilePath, interval));
        const splittedFile2Buffer = yield streamToBufferPromise(_fs2.default.createReadStream(multiSplitted2FilePath, interval));
        const splittedFile3Buffer = yield streamToBufferPromise(_fs2.default.createReadStream(multiSplitted3FilePath, interval));
        const splittedFile4Buffer = yield streamToBufferPromise(_fs2.default.createReadStream(multiSplitted4FilePath, interval));

        t.is(rarFile1Buffer.length, splittedFile1Buffer.length);
        t.is(rarFile2Buffer.length, splittedFile2Buffer.length);
        t.is(rarFile3Buffer.length, splittedFile3Buffer.length);
        t.is(rarFile4Buffer.length, splittedFile4Buffer.length);

        t.deepEqual(rarFile1Buffer, splittedFile1Buffer);
        t.deepEqual(rarFile2Buffer, splittedFile2Buffer);
        t.deepEqual(rarFile3Buffer, splittedFile3Buffer);
        t.deepEqual(rarFile4Buffer, splittedFile4Buffer);
    });

    return function (_x9) {
        return _ref9.apply(this, arguments);
    };
})());