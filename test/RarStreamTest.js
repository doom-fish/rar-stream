import {Readable} from 'stream';
import sinon from 'sinon';
import chai from 'chai';
let expect = chai.expect;
let assert = chai.assert;

chai.should();

import RarStream from '../src/RarStream';

describe('RarStream', () => {
  describe('#constructor', () => {
    it('should be constructable', () => {
      expect(new RarStream()).to.be.an.instanceOf(RarStream);
      expect(new RarStream()).to.be.an.instanceOf(Readable);
    });
  });
});
