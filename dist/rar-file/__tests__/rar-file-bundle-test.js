'use strict';

var _ava = require('ava');

var _ava2 = _interopRequireDefault(_ava);

var _stream = require('stream');

var _fileMedia = require('../../file-media/file-media');

var _fileMedia2 = _interopRequireDefault(_fileMedia);

var _rarFileBundle = require('../rar-file-bundle');

var _rarFileBundle2 = _interopRequireDefault(_rarFileBundle);

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

const newFileMedia = name => new _fileMedia2.default({
    name,
    size: 0,
    createReadStream: () => new _stream.Readable()
});
// eslint-disable-next-line


(0, _ava2.default)('RarFileBundle length should be 0 with an empty array as input', t => {
    const emptyInstance = new _rarFileBundle2.default();
    t.is(emptyInstance.length, 0);
});

(0, _ava2.default)('RarFileBundle should return length with the same length as input', t => {
    const input = ['a.r01', 'a.r02', 'a.r03', 'a.r04', 'a.r05'];
    const fileMedias = input.map(newFileMedia);
    const inputInstance = new _rarFileBundle2.default(...fileMedias);
    t.is(inputInstance.length, input.length);
});

(0, _ava2.default)('RarFileBundle should deconstruct into input', t => {
    const input = ['a.r01', 'a.r02', 'a.r03', 'a.r04', 'a.r05'];
    const fileMedias = input.map(newFileMedia);
    const inputInstance = new _rarFileBundle2.default(...fileMedias);
    t.deepEqual(fileMedias, inputInstance.files);
});

(0, _ava2.default)('RarFileBundle should return unsorted rxx filenames in a sorted manner', t => {
    const unsortedFileNames = ['a.r03', 'a.r02', 'a.rar', 'a.r01', 'a.r00'];
    const fileMedias = unsortedFileNames.map(newFileMedia);
    const sortedFileNames = ['a.rar', 'a.r00', 'a.r01', 'a.r02', 'a.r03'];
    const instanceWithUnsortedParameters = new _rarFileBundle2.default(...fileMedias);
    t.deepEqual(instanceWithUnsortedParameters.fileNames, sortedFileNames);
});

(0, _ava2.default)('RarFileBundle should return unsorted part file names in a sorted manner', t => {
    const sortedFileNames = ['a.part01.rar', 'a.part02.rar', 'a.part03.rar', 'a.part04.rar', 'a.part05.rar', 'a.part06.rar'];

    const unsortedFileNames = ['a.part06.rar', 'a.part01.rar', 'a.part04.rar', 'a.part03.rar', 'a.part05.rar', 'a.part02.rar'];
    const fileMedias = unsortedFileNames.map(newFileMedia);

    const instanceWithUnsortedParameters = new _rarFileBundle2.default(...fileMedias);
    t.deepEqual(instanceWithUnsortedParameters.fileNames, sortedFileNames);
});

(0, _ava2.default)('RarFileBundle should filter out non rar files', t => {
    const unfilteredFileNames = ['a.part01.rar', 'a.part02.rar', 'a.part03.rar', 'a.sfv', 'a.jpg', 'a.part04.rar', 'a.nfo', 'a.part05.rar'];
    const fileMedias = unfilteredFileNames.map(newFileMedia);

    const filteredFileNames = ['a.part01.rar', 'a.part02.rar', 'a.part03.rar', 'a.part04.rar', 'a.part05.rar'];
    const unFilteredInstance = new _rarFileBundle2.default(...fileMedias);
    t.deepEqual(unFilteredInstance.fileNames, filteredFileNames);
});