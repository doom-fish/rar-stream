// @flow
import FileMedia from './file-media'
import fs from 'fs'

export default class LocalFileMedia extends FileMedia {
  constructor (localFilePath: string) {
    if (typeof localFilePath !== 'string') {
      throw new Error('Invalid Arguments, localFilePath' +
                      'need to be passed to the constructor as a string')
    }
    let nameParts = localFilePath.split('/')
    let fileInfo = {
      name: nameParts[nameParts.length - 1],
      size: fs.statSync(localFilePath).size,
      createReadStream: (options) => fs.createReadStream(localFilePath, options)
    }
    super(fileInfo)
  }
}
