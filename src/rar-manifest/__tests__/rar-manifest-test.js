//@flow
import test from 'ava';
import RarManifest from '../rar-manifest';
import RarFileBundle from '../../rar-file/rar-file-bundle'
import MockFileMedia from '../../parsing/__mocks__/mock-file-media';

const files = require('../__fixtures__/files');

function createRarBundle () {
  return new RarFileBundle([new MockFileMedia(files['file.rar'])]);
}

test('file-manifest#getFiles should should a promise', t => {
  t.plan(1);
  const fileBundle = createRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles().then(() => {
    t.pass();
  })
});

test('file-manifest#getFiles should parse rar files and give a list of extractable files', t => {
  t.plan(1);
  const fileBundle = createRarBundle();
  const manifest = new RarManifest(fileBundle);
  return manifest.getFiles().then((files) => {
    t.pass();
  })
});
