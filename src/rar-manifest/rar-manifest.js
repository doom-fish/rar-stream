//@flow
import FileMedia from '../file-media/file-media'
import RarFileBundle from '../rar-file/rar-file-bundle'

export default class RarManifest {
  _rarFileBundle: RarFileBundle;
  constructor(rarFileBundle: RarFileBundle){
    this._rarFileBundle = rarFileBundle;
  }
  getFiles() : Promise<FileMedia[]>{
    return Promise.resolve([new FileMedia({name: 'a'})]);
  }
}
