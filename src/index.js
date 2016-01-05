let RarFileBundle = require("./rar-file-bundle");

module.exports = {
  createRarFileBundle(fileNames) {
    return new RarFileBundle(fileNames);
  }
};
