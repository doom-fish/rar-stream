import test from 'ava';
import sinon from 'sinon';

import FileMedia from '../file-media';

test('FileMedia constructor should throw if options are empty', t => {
  t.throws(() => new FileMedia(), /Invalid Arguments/)
});

test('FileMedia.name should expose the name from the constructor as a getter',  t => {
  const namedTorrentMedia = new FileMedia({name: 'Named Media'});
  t.is(namedTorrentMedia.name, 'Named Media');
});

test('FileMedia.size should expose the size from the constructor as a getter', t => {
    const sizedMedia1 = new FileMedia({size: 1337});
    t.is(sizedMedia1.size, 1337);

    const sizedMedia2 = new FileMedia({size: 5201});
    t.is(sizedMedia2.size, 5201);
});

test('FileMedia.createReadStream should throw if start is greater than end paramter', t => {
  const instance = new FileMedia({});
  t.throws(() => instance.createReadStream(2, 0), /Invalid Arguments/);
});

test('FileMedia.createReadStream should return a promise', t => {

  const instance = new FileMedia({
    createReadStream: sinon.spy(() => ({
        on: (name, cb) => cb()
      }))
  });
  t.truthy(instance.createReadStream(0, 0) instanceof Promise);
});

test('FileMedia.createReadStream should return a readable stream', t => {
  const spy = sinon.spy();
  const torrentFile = {
    createReadStream: sinon.spy(() => ({
        on: (name, cb) => {
          spy(name);
          cb();
        }
      }))
  };
  const instance = new FileMedia(torrentFile);
  return instance.createReadStream(0, 20).then(() => {
    t.deepEqual(torrentFile.createReadStream.args[0], [0, 20]);
    t.truthy(spy.calledWith('readable'));
    t.truthy(spy.calledWith('error'));
  });
});
