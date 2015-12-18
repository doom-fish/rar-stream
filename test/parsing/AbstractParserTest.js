import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import sinon from 'sinon';

chai.use(chaiAsPromised);
let expect = chai.expect;
let assert = chai.assert;

import AbstractParser from '../../src/parsing/AbstractParser';

describe('AbstractParserTest', () => {
  let instance;
  beforeEach(() => {
    instance = new AbstractParser();
  });
  describe('#constructor', () => {
    it('should be constructable', () => {
      expect(instance).to.be.an.instanceof(AbstractParser);
    });
  });
  describe('#parse', () => {
    it('should throw as method is abstract', () => {
      expect(() => instance.parse()).to.throw(/Abstract Method/);
    });
  });
});