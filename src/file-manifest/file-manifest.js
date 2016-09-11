//@flow
import FileMedia from '../file-media/file-media'
import RarFleBundle from '../rar-file-bundle';
export default class FileManifest {
  _rarFileBundle: RarFleBundle;
  constructor(rarFileBundle: RarFleBundle){
    this._rarFileBundle = rarFileBundle;
  }
  getFiles() : Promise<FileMedia[]>{
    return Promise.resolve([new FileMedia({name: 'a'})]);
  }
}
