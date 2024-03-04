# rar-stream

[![TESTS](https://github.com/1313/rar-stream/actions/workflows/test.yml/badge.svg)](https://github.com/1313/rar-stream/actions/workflows/test.yml)

Library for _"unpacking"_ and reading files inside rar archives as node Readable streams.

**Note: Requires node version >= 12.0.0**

**Note: Decompression is not implemented at the moment**

## Getting Started

Below example shows how to unpack local rar files by piping the inner files to the file system.

```javascript
import fs from "fs";
import path from "path";
import { RarFilesPackage, LocalFileMedia } from "rar-stream";
const CWD = process.cwd();

const localRarFiles = [
  path.resolve(CWD, "./file.r00"),
  path.resolve(CWD, "./file.r01"),
  path.resolve(CWD, "./file.r02"),
  path.resolve(CWD, "./file.rar"),
].map((p) => new LocalFileMedia(p));

const rarFilesPackage = new RarFilesPackage(localRarFiles);

async function writeInnerRarFilesToDisk() {
  const innerFiles = await rarFilesPackage.parse();
  for (const innerFile of innerFiles) {
    innerFile
      .createReadStream({ start: 0, end: innerFile.length - 1 })
      .pipe(fs.createWriteStream(innerFile.name));
  }
}

await writeInnerRarFilesToDisk();
```

See [example/webtorrent.js](example/webtorrent.js) for a more advanced example.

### Installing

Install from npm repo with:

```
npm i rar-stream
```

## API

### RarFilesPackage Api

#### Methods:

| Method        | Description                                                                                                                                 |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| _constructor_ | Takes an array of local file paths as strings or instances that satifies the [`FileMedia`](#filemedia-interface) interface mentioned below. |
| parse         | Parses all rar files and returns a Promise with [`InnerFile`](#innerfile-api)s.                                                             |

#### Events:

| Event            | Description                                                                                                                                               |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| parsing-start    | Emitted when the parsing is started, happens when you call `parse`. Event args are a bundle represntation of all the rar files passed to the constructor. |
| file-parsed      | Emitted each time a rar file is parsed. The event argument is the RarFile just parsed, i.e `.rxx` in the chain.                                           |
| parsing-complete | Emitted when the parsing is completed. The event argument is an array of all the parsed [`InnerFile`](#innerfile-api)s.                                   |

#### Example

```
const rarFilesPackage = new RarFilesPackage(localRarFiles);
rarFilesPackage.on('parsing-start', rarFiles => console.log(rarFiles))
rarFilesPackage.on('file-parsed', rarFile => console.log(rarFile.name))
rarFilesPackage.on('parsing-end', innerFiles => console.log(innerFiles))
const innerFiles = await rarFilesPackage.parse();
```

### InnerFile Api

Implements the [`FileMedia`](#filemedia-interface) interface.

#### Methods:

| Method                                         | Description                                                             |
| ---------------------------------------------- | ----------------------------------------------------------------------- |
| createReadStream({start: number, end: number}) | Returns a `Readable` stream. The start and end interval is inclusive.   |
| readToEnd                                      | Returns a Promise with a Buffer containing all the content of the file. |

#### Properties:

| Property | Description                                   |
| -------- | --------------------------------------------- |
| name     | The name of the file                          |
| length   | Returns the total number of bytes of the file |

#### Example

```
const innerFiles = await rarStreamPackage.parse();
const innerFileStream = innerFiles[0].createReadStream({ start: 0, end: 30});
```

### _FileMedia Interface_

This is loosely enforced interface that makes this module interoptable with other node modules such as [`torrent-stream`](https://www.npmjs.com/package/torrent-stream) or [`webtorrent`](https://www.npmjs.com/package/webtorrent).

Should have the following shape:

```javascript
 // FileMedia
 {
  createReadStream(interval: Interval): Readable,
  name: string,
  length: number // Length or size of the file in bytes
 }

 // Interval
 // start and end should be inclusive.
 {
  start: number,
  end: number
 }
```

## Development

### Running the tests

Run tests with:

```
npm test
```

### Contributing

Post a new issue if you'd like to contribute in any way.

### Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/1313/rar-stream/tags).

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
