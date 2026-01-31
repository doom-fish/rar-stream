/**
 * rar-stream browser entry point with Web Streams API support
 */

// Re-export WASM bindings
export {
  default as init,
  initSync,
  isRarArchive,
  getRarVersion,
  parseRarHeader,
  RarDecoder,
  WasmRar5Crypto,
  is_rar_archive,
  get_rar_version,
  parse_rar_header,
  WasmRarDecoder,
} from '../browser.js';

/** Options for creating a ReadableStream */
export interface ReadableStreamOptions {
  /** Total size of the data */
  totalSize: number;
  /** Start offset (default: 0) */
  start?: number;
  /** End offset inclusive (default: totalSize - 1) */
  end?: number;
  /** Size of each chunk to read (default: 65536) */
  chunkSize?: number;
  /** Function to read a chunk of data */
  readChunk: (start: number, end: number) => Promise<Uint8Array>;
}

/** Options for creating a range response */
export interface RangeResponseOptions {
  /** Total file size */
  totalSize: number;
  /** HTTP Range header value */
  rangeHeader?: string;
  /** MIME type (default: 'application/octet-stream') */
  contentType?: string;
  /** Function to read a chunk of data */
  readChunk: (start: number, end: number) => Promise<Uint8Array>;
}

/** Result from createRangeResponse */
export interface RangeResponseResult {
  stream: ReadableStream<Uint8Array>;
  headers: Headers;
  status: number;
}

/**
 * Create a Web ReadableStream from an async data source.
 * Useful for Service Workers and streaming responses.
 */
export declare function createReadableStream(options: ReadableStreamOptions): ReadableStream<Uint8Array>;

/**
 * Helper to create a streaming response for HTTP range requests.
 * Parses the Range header and returns appropriate stream, headers, and status.
 */
export declare function createRangeResponse(options: RangeResponseOptions): RangeResponseResult;
