let sortRxx = Symbol();
let sortPartxx = Symbol();
let calculateExtensionType = Symbol();
let fileNamesProperty = Symbol();

let rxxExtension =  /\.r(\d\d)$|.rar$/;
let rarExtension = /.rar$/;
let partRxxExtension = /.part(\d\d).rar/;

export default class RarFileBundle {
  constructor(fileNames){
    if(!fileNames){
      throw new Error("Invalid Arguments, fileNames need to be passed to the constructor");
    }
    this[fileNamesProperty] = fileNames;
    this[calculateExtensionType]();
    
    this.filter();
    this.sort();
  }
  [calculateExtensionType](){
    let anyPartXXTypes = this[fileNamesProperty].filter(part => part.match(partRxxExtension));
    
    if(anyPartXXTypes.length > 0){
      this.extensionType = "partxx";
    }else{
      this.extensionType =  "rxx"
    }
  }
  filter(){
    if(this.extensionType === "rxx"){
      this[fileNamesProperty] = this[fileNamesProperty].filter(part => part.match(rxxExtension));
    }else{
      this[fileNamesProperty] = this[fileNamesProperty].filter(part => part.match(partRxxExtension));
    }
  }
  sort(){
    if(this.extensionType === "rxx"){
      this[sortRxx]();
    }else{
      this[sortPartxx]();
    }
  }
  [sortPartxx](){
    this[fileNamesProperty].sort((first, second) => {
      let firstMatch = first.match(partRxxExtension);
      let secondMatch = second.match(partRxxExtension);
      let firstNumber = +(firstMatch && firstMatch[1] || 0);
      let secondNumber = +(secondMatch && secondMatch[1] || 0);
      return  firstNumber - secondNumber; 
    });
  }
  [sortRxx](){
    this[fileNamesProperty].sort((first, second) => {
      if (first.match(rarExtension)) {
        return -1;
      } else if (second.match(rarExtension)) {
        return 1;
      } else {
        let firstMatch = first.match(rxxExtension);
        let secondMatch = second.match(rxxExtension);
        let firstNumber = +(firstMatch && firstMatch[1] || 0);
        let secondNumber = +(secondMatch && secondMatch[1] || 0);
        return  firstNumber - secondNumber;
      }
    });
  }
  get length(){
    return this[fileNamesProperty].length;
  }
  *[Symbol.iterator] (){
    yield* this[fileNamesProperty];
  }
};