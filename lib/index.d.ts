/**
 * rar-stream - Node.js wrapper with Readable stream support
 */

import { Readable } from 'stream';

/** Read interval options. */
export interface ReadIntervalJs {
  start: number;
  end: number;
}

/** Stream options with optional highWaterMark. */
export interface StreamOptions {
  start?: number;
  end?: number;
  highWaterMark?: number;
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
 * Parse RAR file header from a buffer.
 * This can be used to detect RAR archives and get inner file info
 * without downloading the entire archive.
 *
 * The buffer should contain at least the first ~300 bytes of a .rar file.
 */
export declare function parseRarHeader(buffer: Buffer): RarFileInfo | null;

/** Check if a buffer starts with a RAR signature. */
export declare function isRarArchive(buffer: Buffer): boolean;

/** LocalFileMedia - reads from local filesystem. */
export declare class LocalFileMedia {
  constructor(path: string);
  
  readonly name: string;
  readonly length: number;
  
  /** Read a byte range and return as Buffer. */
  createReadStream(opts: ReadIntervalJs): Promise<Buffer>;
  
  /**
   * Get a Node.js Readable stream for a byte range.
   * Use this for streaming without loading the entire range into memory.
   */
  getReadableStream(opts: StreamOptions & { start: number; end: number }): Readable;
}

/**
 * InnerFile - a file inside the RAR archive.
 * Provides both buffer-based and stream-based access.
 */
export declare class InnerFile {
  readonly name: string;
  readonly length: number;
  
  /** Read a byte range and return as Buffer. */
  createReadStream(opts: ReadIntervalJs): Promise<Buffer>;
  
  /** Read entire file into memory. */
  readToEnd(): Promise<Buffer>;
  
  /**
   * Get a Node.js Readable stream for the entire file or a byte range.
   * 
   * @example
   * // Stream entire file
   * const stream = file.getReadableStream();
   * stream.pipe(fs.createWriteStream('output.bin'));
   * 
   * @example
   * // Stream with range (for HTTP range requests, WebTorrent, etc.)
   * const stream = file.getReadableStream({ start: 0, end: 1024 * 1024 - 1 });
   */
  getReadableStream(opts?: StreamOptions): Readable;
}

/** RarFilesPackage - parses multi-volume RAR archives. */
export declare class RarFilesPackage {
  constructor(files: LocalFileMedia[]);
  
  /** Parse the archive and return inner files. */
  parse(opts?: ParseOptionsJs | undefined | null): Promise<InnerFile[]>;
}
