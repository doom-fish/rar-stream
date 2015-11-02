import chai from 'chai';
let expect = chai.expect;
chai.should();

import RarFileBundle from '../src/RarFileBundle';

describe('RarFileBundle', () => {
  let instance;
  beforeEach(() => {
    instance = new RarFileBundle([]);
  });
  describe('#constructor', () => {
    it('should be constructable', () => {
      expect(instance).to.be.an.instanceOf(RarFileBundle);
    });
    it('should take array of strings as first parameter', () => {
      expect(() => new RarFileBundle()).to.throw(/Invalid Arguments/);
    });
  });
  describe('#length', () => {
    it('should return lenght should be defined', () => { 
      expect(instance.length).to.not.be.undefined;
    });
    it('should return lenght should be 0 with an empty array as input', () => { 
      let emptyInstance = new RarFileBundle([]);
      expect(emptyInstance.length).to.equal(0);
    });
    it('should return lenght should be same length as input', () => { 
      let input = ["a.r01","a.r02","a.r03","a.r04","a.r05"];
      let inputInstance = new RarFileBundle(input);
      expect(inputInstance.length).to.equal(input.length);
    });
  });
  describe('#iterator', () => {
    it('should be defined', () => {
      expect(instance[Symbol.iterator]).to.not.be.undefined;
    });
    it('should deconstruct into input parameteres', () => {
      let input = ["a.r01","a.r02","a.r03","a.r04"];
      let toDeconstruct = new RarFileBundle(input);
      expect([...toDeconstruct]).to.be.eql(input);
    });
  });
  describe('#sort functionality', () => {
    it('should return unsorted rxx file names in a sorted manner', () =>{
      let unsortedFileNames = ['a.r03', 'a.r02', 'a.rar', 'a.r01', 'a.r00'];
      let sortedFileNames = ['a.rar', 'a.r00', 'a.r01', 'a.r02', 'a.r03'];
      let instanceWithUnsortedParameters = new RarFileBundle(unsortedFileNames);
      expect([...instanceWithUnsortedParameters]).to.be.eql(sortedFileNames);
    });
    it('should return unsorted part file names in a sorted manner', () =>{
      let sortedFileNames = [
        'a.part01.rar',
        'a.part02.rar',
        'a.part03.rar',
        'a.part04.rar',
        'a.part05.rar',
        'a.part06.rar'
      ];

      let unsortedFileNames = [
        'a.part06.rar',
        'a.part01.rar',
        'a.part04.rar',
        'a.part03.rar',
        'a.part05.rar',
        'a.part02.rar'
      ];

      let instanceWithUnsortedParameters = new RarFileBundle(unsortedFileNames);
      expect([...instanceWithUnsortedParameters]).to.be.eql(sortedFileNames);
    });
  });
  describe('#filter functionality', () => {
    it('should filter out non rar files', () => {
      let unfilteredFileNames = [
        'a.part01.rar',
        'a.part02.rar',
        'a.part03.rar',
        'a.sfv',
        'a.jpg',
        'a.part04.rar',
        'a.nfo',
        'a.part05.rar'
      ];

      let filteredFileNames = [
        'a.part01.rar',
        'a.part02.rar',
        'a.part03.rar',
        'a.part04.rar',
        'a.part05.rar'
      ];
      let unFilteredInstance = new RarFileBundle(unfilteredFileNames);
      expect([...unFilteredInstance]).to.be.eql(filteredFileNames);
    })
  })
});
