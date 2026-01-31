/**
 * rar-stream - Node.js wrapper with Readable stream support
 */

import { Readable } from 'stream';

/** Read interval options. */
export interface ReadIntervalJs {
  start: number;
  end: number;
}

/** Stream options for createReadStream. */
export interface StreamOptions {
  start?: number;
  end?: number;
}

/** Parse options for filtering results. */
export interface ParseOptionsJs {
  maxFiles?: number;
}

/** Parsed file info from RAR header. */
export interface RarFileInfo {
  name: string;
  packedSize: number;
  unpackedSize: number;
  method: number;
  continuesInNext: boolean;
}

/**
 * FileMedia interface for custom file sources.
 * Implement this to use WebTorrent, HTTP, S3, etc.
 */
export interface FileMedia {
  readonly name: string;
  readonly length: number;
  createReadStream(opts: ReadIntervalJs): Readable;
}

/**
 * Helper to read a Readable stream into a Buffer.
 */
export declare function streamToBuffer(stream: Readable): Promise<Buffer>;

/**
 * Create a FileMedia wrapper from any object with createReadStream.
 * Use this to wrap WebTorrent files, HTTP responses, S3 objects, etc.
 */
export declare function createFileMedia(source: FileMedia): FileMedia;

/**
 * Parse RAR file header from a buffer.
 * This can be used to detect RAR archives and get inner file info
 * without downloading the entire archive.
 *
 * The buffer should contain at least the first ~300 bytes of a .rar file.
 */
export declare function parseRarHeader(buffer: Buffer): RarFileInfo | null;

/** Check if a buffer starts with a RAR signature. */
export declare function isRarArchive(buffer: Buffer): boolean;

/**
 * LocalFileMedia - reads from local filesystem.
 * Implements the FileMedia interface.
 */
export declare class LocalFileMedia implements FileMedia {
  constructor(path: string);
  
  readonly name: string;
  readonly length: number;
  
  /**
   * Create a Readable stream for a byte range.
   * @param opts Byte range (inclusive)
   */
  createReadStream(opts: ReadIntervalJs): Readable;
}

/**
 * InnerFile - a file inside the RAR archive.
 * Provides stream-based access to file content.
 */
export declare class InnerFile {
  readonly name: string;
  readonly length: number;
  
  /**
   * Create a Readable stream for the entire file or a byte range.
   * 
   * @example
   * // Stream entire file
   * const stream = file.createReadStream();
   * stream.pipe(fs.createWriteStream('output.bin'));
   * 
   * @example
   * // Stream with range (for HTTP range requests, WebTorrent, etc.)
   * const stream = file.createReadStream({ start: 0, end: 1024 * 1024 - 1 });
   */
  createReadStream(opts?: StreamOptions): Readable;
  
  /** Read entire file into memory. */
  readToEnd(): Promise<Buffer>;
}

/**
 * RarFilesPackage - parses multi-volume RAR archives.
 * 
 * Supports both LocalFileMedia and custom FileMedia implementations.
 */
export declare class RarFilesPackage {
  constructor(files: FileMedia[]);
  
  /** Parse the archive and return inner files. */
  parse(opts?: ParseOptionsJs | undefined | null): Promise<InnerFile[]>;
}
