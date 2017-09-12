const test = require('ava');
const { Readable } = require('stream');

const makeRarFileBundle = require('../rar-file-bundle');

const newFileMedia = name => ({
    name,
    size: 0,
    createReadStream: () => new Readable(),
});

test('RarFileBundle length should be 0 with an empty array as input', t => {
    const emptyInstance = makeRarFileBundle();
    t.is(emptyInstance.length, 0);
});

test('RarFileBundle should return length with the same length as input', t => {
    const input = ['a.r01', 'a.r02', 'a.r03', 'a.r04', 'a.r05'];
    const fileMedias = input.map(newFileMedia);
    const inputInstance = makeRarFileBundle(fileMedias);
    t.is(inputInstance.length, input.length);
});

test('RarFileBundle should deconstruct into input', t => {
    const input = ['a.r01', 'a.r02', 'a.r03', 'a.r04', 'a.r05'];
    const fileMedias = input.map(newFileMedia);
    const inputInstance = makeRarFileBundle(fileMedias);
    t.deepEqual(fileMedias, inputInstance.files);
});

test('RarFileBundle should return unsorted rxx filenames in a sorted manner', t => {
    const unsortedFileNames = ['a.r03', 'a.r02', 'a.rar', 'a.r01', 'a.r00'];
    const fileMedias = unsortedFileNames.map(newFileMedia);
    const sortedFileNames = ['a.rar', 'a.r00', 'a.r01', 'a.r02', 'a.r03'];
    const instanceWithUnsortedParameters = makeRarFileBundle(fileMedias);
    t.deepEqual(instanceWithUnsortedParameters.fileNames, sortedFileNames);
});

test('RarFileBundle should return unsorted part file names in a sorted manner', t => {
    const sortedFileNames = [
        'a.part01.rar',
        'a.part02.rar',
        'a.part03.rar',
        'a.part04.rar',
        'a.part05.rar',
        'a.part06.rar',
    ];

    const unsortedFileNames = [
        'a.part06.rar',
        'a.part01.rar',
        'a.part04.rar',
        'a.part03.rar',
        'a.part05.rar',
        'a.part02.rar',
    ];
    const fileMedias = unsortedFileNames.map(newFileMedia);

    const instanceWithUnsortedParameters = makeRarFileBundle(fileMedias);
    t.deepEqual(instanceWithUnsortedParameters.fileNames, sortedFileNames);
});

test('RarFileBundle should filter out non rar files', t => {
    const unfilteredFileNames = [
        'a.part01.rar',
        'a.part02.rar',
        'a.part03.rar',
        'a.sfv',
        'a.jpg',
        'a.part04.rar',
        'a.nfo',
        'a.part05.rar',
    ];
    const fileMedias = unfilteredFileNames.map(newFileMedia);

    const filteredFileNames = [
        'a.part01.rar',
        'a.part02.rar',
        'a.part03.rar',
        'a.part04.rar',
        'a.part05.rar',
    ];
    const unFilteredInstance = makeRarFileBundle(fileMedias);
    t.deepEqual(unFilteredInstance.fileNames, filteredFileNames);
});
