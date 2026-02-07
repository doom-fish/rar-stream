/**
 * rar-stream browser entry point with Web Streams API support
 */

// WASM init
/**
 * Initialize the WASM module. Safe to call multiple times.
 * Automatically called by all async helper functions.
 */
export declare function init(wasmUrl?: string | URL | Request | Response | BufferSource | WebAssembly.Module): Promise<void>;
export declare function initSync(module: { module: BufferSource | WebAssembly.Module } | BufferSource | WebAssembly.Module): void;

// Auto-init async helpers (call init() automatically)
export declare function isRarArchive(data: Uint8Array): Promise<boolean>;
export declare function getRarVersion(data: Uint8Array): Promise<number>;
export declare function parseRarHeader(data: Uint8Array): Promise<any>;
export declare function parseRarHeaders(data: Uint8Array): Promise<any[]>;
export declare function parseRar5Header(data: Uint8Array): Promise<any>;
export declare function parseRar5Headers(data: Uint8Array): Promise<any[]>;

/**
 * Extract the first file from a RAR archive buffer.
 * Auto-detects RAR4/RAR5, parses headers, and decompresses in one call.
 */
export declare function extractFile(data: Uint8Array): Promise<{name: string, data: Uint8Array, length: number}>;

// Classes (require init() before construction)
export declare class RarFilesPackage {
  constructor(data: Uint8Array);
  readonly length: number;
  parse(): Array<{name: string, length: number, packedSize: number, isDirectory: boolean}>;
  extract(index: number): {name: string, data: Uint8Array, length: number};
  extractAll(): Array<{name: string, data: Uint8Array, length: number}>;
  free(): void;
}

export declare class RarDecoder {
  constructor(unpackedSize: bigint);
  decompress(data: Uint8Array): Uint8Array;
  bytes_written(): bigint;
  is_complete(): boolean;
  reset(): void;
  free(): void;
}

export declare class Rar5Decoder {
  constructor(unpackedSize: bigint, dictSizeLog: number, method: number, isSolid: boolean);
  decompress(data: Uint8Array): Uint8Array;
  reset(): void;
  free(): void;
}

export declare class Rar5Crypto {
  constructor(password: string, salt: Uint8Array, lg2Count: number);
  decrypt(iv: Uint8Array, data: Uint8Array): Uint8Array;
  verify_password(checkValue: Uint8Array): boolean;
  free(): void;
}

// Direct snake_case access (require init() before use)
export {
  is_rar_archive,
  get_rar_version,
  parse_rar_header,
  parse_rar_headers,
  parse_rar5_header,
  parse_rar5_headers,
  extract_file,
} from '../pkg/rar_stream.d.ts';

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
