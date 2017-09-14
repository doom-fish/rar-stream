# rar-stream
[![Build Status](https://api.travis-ci.org/1313/rar-stream.svg)](https://travis-ci.org/1313/rar-stream)

Library for _"unpacking"_ and reading files inside rar archives as node Readable streams.

__Note: Requires node version >= 8.0.0__

__Note: Uncompression is not implemented at the moment__

## Getting Started
Below example shows howto unpack local rar files by piping the inner files to the file system.

```javascript
const fs = require('fs');
const path = require('path');
const { RarFilesPackage } = require('rar-stream');
const CWD = process.cwd();
const localRarFiles = [
  path.resolve(CWD, './file.r00'),
  path.resolve(CWD, './file.r01'),
  path.resolve(CWD, './file.r02'),
  path.resolve(CWD, './file.rar')
];

const rarFilesPackage = new RarFilesPackage(localRarFiles);

async function writeInnerRarFilesToDisk() {
    const innerFiles = await rarFilesPackage.parse();
    for(const innerFile of innerFiles) {
      innerFile
        .createReadStream({ start: 0, end: innerFile.length - 1})
        .pipe(fs.createWriteStream(innerFile.name));
    }
}

writeInnerRarFilesToDisk();

```
See [example/webtorrent.js](example/webtorrent.js) for a more advanced example.

### Installing

Install from npm repo with:

```
npm i rar-stream
```

## API

### RarFilesPackage Api

```
// example
const rarFilesPackage = new RarFilesPackage(localRarFiles);
rarFilesPackage.on('parsing-start', rarFiles => console.log(rarFiles))
rarFilesPackage.on('file-parsed', innerFile => console.log(innerFile.name))
rarFilesPackage.on('parsing-end', innerFiles => console.log(innerFiles))
const innerFiles = await rarFilesPackage.parse();
```

#### Methods:
Method | Description
------|------------
_constructor_ | Takes an array of local file paths as strings or instances that satifies the [`FileMedia`](#filemedia-interface) api mentioned below.
parse | Parses all rar files and returns a Promise with `InnerFile`s.

#### Events:
Event | Description
------|------------
parsing-start | Emitted when the parsing is started, happens when you call `parse`. Event args are a bundle represntation of all the rar files passed to the constructor.
file-parsed | Emitted each time a rar file is parsed. The event argument is the RarFile just parsed, i.e `.rxx` in the chain.
parsing-complete | Emitted when the parsing is completed. The event argument is an array of all the parsed [`InnerFile`](#innerfile-api)s.

### InnerFile Api
Implements the [`FileMedia`](#filemedia-interface) interface.
```
// example
const innerFiles = await rarStreamPackage.parse();
const innerFileStream = innerFiles[0].createReadStream(0, 10);
```
#### Methods:
Method | Description
------|------------
createReadStream(start: number, end: number) | Returns a `Readable` stream. The start and end interval is inclusive.
readToEnd | Returns a Promise with a Buffer containing all the content of the file.

#### Properties:
Property | Description
------|------------
name | The name of the file
length | Returns the number of bytes

### _FileMedia Interface_
This is loosely enforced interface that makes this module interoptable with other node modules such as [`torrent-stream`](https://www.npmjs.com/package/torrent-stream) or [`webtorrent`](https://www.npmjs.com/package/webtorrent). 

Should have the following shape:
```javascript
 {
  createReadStream(start: number, end: number): Readable, // start and end interval should be inclusive.
  name: string,
  length: number // Length or size of the file in bytes
 }
```


## Development

### Running the tests

Run tests with:
```
npm test
```

### Built With

* [binary](https://www.npmjs.com/package/binary) - For parsing rar headers.

### Contributing

Post a new issue if you'd like to contribute in any way.

### Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/1313/rar-stream/tags). 

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details

