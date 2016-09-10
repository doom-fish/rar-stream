//@flow
import test from 'ava';
import td from 'testdouble';

import FileMedia from '../file-media';

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
  const createReadStream = td.function();
  td.when(createReadStream(0,0)).thenReturn({
    on: (name, cb) => {
      cb();
    }
  });

  const instance = new FileMedia({
    createReadStream: createReadStream
  });

  t.truthy(instance.createReadStream(0, 0) instanceof Promise);
});

test('FileMedia.createReadStream should return a readable stream', () => {
  const eventSubscribed = td.function('.eventSubscribed');
  const createReadStream = td.function();

  const stream = {
    on: (name, cb) => {
      eventSubscribed(name);
      cb();
    }
  };

  td.when(createReadStream(
    td.matchers.isA(Number),
    td.matchers.isA(Number)
  )).thenReturn(stream);

  const torrentFile = {createReadStream};
  const instance = new FileMedia(torrentFile);

  return instance.createReadStream(0, 20).then(() => {
    td.verify(eventSubscribed('readable'));
    td.verify(eventSubscribed('error'));
  });
});
