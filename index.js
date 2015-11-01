var RarFileBundle = require('./RarFileBundle');

function createRarFileBundle(fileNames){
  return new RarFileBundle(fileNames);
}



module.exports = {
  createRarFileBundle: createRarFileBundle
}