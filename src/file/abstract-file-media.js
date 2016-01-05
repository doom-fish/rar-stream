export default class AbstractFileMedia {
  createReadStream() {
    return new Promise((resolve, reject) => {
      reject(Error("Abstract Method, make sure to implement this method in sub class"));
    });
  }
  get size() {
    throw Error("Abstract Method, make sure to implement this method in sub class");
  }
  get name() {
    throw Error("Abstract Method, make sure to implement this method in sub class");
  }
}
