'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _rarStream = require('../rar-stream');

var _rarStream2 = _interopRequireDefault(_rarStream);

var _rarFileChunk = require('../rar-file-chunk');

var _rarFileChunk2 = _interopRequireDefault(_rarFileChunk);

var _mockFileMedia = require('../../parsing/__mocks__/mock-file-media');

var _mockFileMedia2 = _interopRequireDefault(_mockFileMedia);

var _streamToBuffer = require('stream-to-buffer');

var _streamToBuffer2 = _interopRequireDefault(_streamToBuffer);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

function _asyncToGenerator(fn) { return function () { var gen = fn.apply(this, arguments); return new Promise(function (resolve, reject) { function step(key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { return Promise.resolve(value).then(function (value) { step("next", value); }, function (err) { step("throw", err); }); } } return step("next"); }); }; }

const streamToBufferPromise = stream => new Promise((resolve, reject) => (0, _streamToBuffer2.default)(stream, (err, buffer) => err ? reject(err) : resolve(buffer)));

(0, _ava2.default)('rar stream should stream over list of file chunks', (() => {
    var _ref = _asyncToGenerator(function* (t) {
        const bufferString = '123456789ABC';
        const fileMedia = new _mockFileMedia2.default(bufferString);

        const rarStream = new _rarStream2.default([new _rarFileChunk2.default(fileMedia, 0, 2), new _rarFileChunk2.default(fileMedia, 2, 6)]);
        const buffer = yield streamToBufferPromise(rarStream);
        t.deepEqual(buffer, new Buffer(bufferString, 'hex'));
    });

    return function (_x) {
        return _ref.apply(this, arguments);
    };
})());

(0, _ava2.default)('rar stream should stream over list of file chunks that are fragmented', (() => {
    var _ref2 = _asyncToGenerator(function* (t) {
        const bufferString = '123456789ABC';
        const fragmentedResult = '349ABC';
        const fileMedia = new _mockFileMedia2.default(bufferString);

        const rarStream = new _rarStream2.default([new _rarFileChunk2.default(fileMedia, 1, 2), new _rarFileChunk2.default(fileMedia, 4, 6)]);
        const buffer = yield streamToBufferPromise(rarStream);
        t.deepEqual(buffer, new Buffer(fragmentedResult, 'hex'));
    });

    return function (_x2) {
        return _ref2.apply(this, arguments);
    };
})());

(0, _ava2.default)('rar stream should stream over longer list of file chunks', (() => {
    var _ref3 = _asyncToGenerator(function* (t) {
        const bufferString = '123456789ABC';
        const fileMedia = new _mockFileMedia2.default(bufferString);

        const rarStream = new _rarStream2.default([new _rarFileChunk2.default(fileMedia, 0, 2), new _rarFileChunk2.default(fileMedia, 2, 4), new _rarFileChunk2.default(fileMedia, 4, 6)]);

        const buffer = yield streamToBufferPromise(rarStream);
        t.deepEqual(buffer, new Buffer(bufferString, 'hex'));
    });

    return function (_x3) {
        return _ref3.apply(this, arguments);
    };
})());