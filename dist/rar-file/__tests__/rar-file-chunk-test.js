'use strict';

var _streamToBuffer = require('stream-to-buffer');

var _streamToBuffer2 = _interopRequireDefault(_streamToBuffer);

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _mockFileMedia = require('../../parsing/__mocks__/mock-file-media');

var _mockFileMedia2 = _interopRequireDefault(_mockFileMedia);

var _rarFileChunk = require('../rar-file-chunk');

var _rarFileChunk2 = _interopRequireDefault(_rarFileChunk);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

function _asyncToGenerator(fn) { return function () { var gen = fn.apply(this, arguments); return new Promise(function (resolve, reject) { function step(key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { return Promise.resolve(value).then(function (value) { step("next", value); }, function (err) { step("throw", err); }); } } return step("next"); }); }; }

const streamToBufferPromise = stream => new Promise((resolve, reject) => (0, _streamToBuffer2.default)(stream, (err, buffer) => err ? reject(err) : resolve(buffer)));

(0, _ava2.default)('RarFileChunk#getStream should return a stream from its FileMedia', (() => {
    var _ref = _asyncToGenerator(function* (t) {
        const bufferString = '123456789A';
        const fileMedia = new _mockFileMedia2.default(bufferString);
        const rarFileChunk = new _rarFileChunk2.default(fileMedia, 0, 5);
        const stream = yield rarFileChunk.getStream();
        const buffer = yield streamToBufferPromise(stream);
        t.deepEqual(new Buffer(bufferString, 'hex'), buffer);
    });

    return function (_x) {
        return _ref.apply(this, arguments);
    };
})());

(0, _ava2.default)('RarFileChunk#getStream should return a stream with a subset stream of FileMedia', (() => {
    var _ref2 = _asyncToGenerator(function* (t) {
        const bufferString = '123456789A';
        const fileMedia = new _mockFileMedia2.default(bufferString);
        const rarFileChunk = new _rarFileChunk2.default(fileMedia, 2, 5);
        const stream = yield rarFileChunk.getStream();
        const buffer = yield streamToBufferPromise(stream);
        t.deepEqual(new Buffer('56789A', 'hex'), buffer);
    });

    return function (_x2) {
        return _ref2.apply(this, arguments);
    };
})());

(0, _ava2.default)('RarFileChunk#getStream should return a stream with another subset stream of FileMedia', (() => {
    var _ref3 = _asyncToGenerator(function* (t) {
        const bufferString = '123456789A';
        const fileMedia = new _mockFileMedia2.default(bufferString);
        const rarFileChunk = new _rarFileChunk2.default(fileMedia, 1, 3);
        const stream = yield rarFileChunk.getStream();
        const buffer = yield streamToBufferPromise(stream);
        t.deepEqual(new Buffer('3456', 'hex'), buffer);
    });

    return function (_x3) {
        return _ref3.apply(this, arguments);
    };
})());

(0, _ava2.default)('RarFileChunk#length should return end - start offset', t => {
    const bufferString = '123456789A';
    const fileMedia = new _mockFileMedia2.default(bufferString);
    let rarFileChunk = new _rarFileChunk2.default(fileMedia, 1, 3);
    t.is(rarFileChunk.length, 2);
    rarFileChunk = new _rarFileChunk2.default(fileMedia, 0, 3);
    t.is(rarFileChunk.length, 3);
    rarFileChunk = new _rarFileChunk2.default(fileMedia, 1, 2);
    t.is(rarFileChunk.length, 1);
    rarFileChunk = new _rarFileChunk2.default(fileMedia, 0, 5);
    t.is(rarFileChunk.length, 5);
});