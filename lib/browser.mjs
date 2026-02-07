/**
 * rar-stream browser entry point with Web Streams API support
 * 
 * Provides ReadableStream support for streaming file content in browsers.
 */

import wasmInit, {
  initSync as _initSync,
  is_rar_archive,
  get_rar_version,
  parse_rar_header,
  parse_rar_headers,
  parse_rar5_header,
  parse_rar5_headers,
  extract_file,
  WasmRarArchive,
  WasmRarDecoder,
  WasmRar5Decoder,
  WasmRar5Crypto,
} from '../pkg/rar_stream.js';

let _initialized = false;
let _initPromise = null;

/**
 * Initialize the WASM module. Safe to call multiple times — subsequent calls are no-ops.
 * Automatically called by all exported functions, but can be called explicitly for eager loading.
 * @param {string | URL | Request | Response | BufferSource | WebAssembly.Module} [wasmUrl] - Optional WASM source
 */
export async function init(wasmUrl) {
  if (_initialized) return;
  if (_initPromise) return _initPromise;
  _initPromise = wasmInit(wasmUrl).then(() => { _initialized = true; });
  return _initPromise;
}

export { _initSync as initSync };

// Auto-init wrappers for all functions
async function ensureInit() {
  if (!_initialized) await init();
}

/** Check if a buffer contains a RAR signature. */
export async function isRarArchive(data) {
  await ensureInit();
  return is_rar_archive(data);
}

/** Get the RAR format version (15 for RAR4, 50 for RAR5, 0 if not RAR). */
export async function getRarVersion(data) {
  await ensureInit();
  return get_rar_version(data);
}

/** Parse the first RAR4 file header from a buffer. */
export async function parseRarHeader(data) {
  await ensureInit();
  return parse_rar_header(data);
}

/** Parse all RAR4 file headers from a buffer. */
export async function parseRarHeaders(data) {
  await ensureInit();
  return parse_rar_headers(data);
}

/** Parse the first RAR5 file header from a buffer. */
export async function parseRar5Header(data) {
  await ensureInit();
  return parse_rar5_header(data);
}

/** Parse all RAR5 file headers from a buffer. */
export async function parseRar5Headers(data) {
  await ensureInit();
  return parse_rar5_headers(data);
}

// Re-export classes (require manual init() before use)
export { WasmRarArchive, WasmRarDecoder, WasmRar5Decoder, WasmRar5Crypto };
// Aliases
export { WasmRarArchive as RarArchive, WasmRarDecoder as RarDecoder, WasmRar5Decoder as Rar5Decoder };

/**
 * Extract the first file from a RAR archive buffer.
 * Auto-detects RAR4/RAR5, parses headers, and decompresses in one call.
 * @param {Uint8Array} data - The entire RAR archive
 * @returns {Promise<{name: string, data: Uint8Array, size: number}>}
 */
export async function extractFile(data) {
  await ensureInit();
  return extract_file(data);
}

// Re-export snake_case direct access (require manual init() before use)
export {
  is_rar_archive,
  get_rar_version,
  parse_rar_header,
  parse_rar_headers,
  parse_rar5_header,
  parse_rar5_headers,
  extract_file,
};

/**
 * Create a Web ReadableStream from an async data source.
 * 
 * This utility helps create streaming responses for browsers,
 * useful for Service Workers and fetch handlers.
 * 
 * @param {Object} options
 * @param {number} options.totalSize - Total size of the data
 * @param {number} [options.start=0] - Start offset
 * @param {number} [options.end] - End offset (inclusive), defaults to totalSize-1
 * @param {number} [options.chunkSize=65536] - Size of each chunk to read
 * @param {function(start: number, end: number): Promise<Uint8Array>} options.readChunk - Function to read a chunk
 * @returns {ReadableStream<Uint8Array>}
 * 
 * @example
 * // In a Service Worker fetch handler
 * const stream = createReadableStream({
 *   totalSize: file.length,
 *   start: rangeStart,
 *   end: rangeEnd,
 *   readChunk: async (start, end) => {
 *     const decoder = new WasmRarDecoder(file.unpackedSize);
 *     // ... fetch and decompress data
 *     return decompressedData.slice(start, end + 1);
 *   }
 * });
 * return new Response(stream, { headers: { 'Content-Type': 'video/mp4' } });
 */
export function createReadableStream(options) {
  const { totalSize, start = 0, end = totalSize - 1, chunkSize = 64 * 1024, readChunk } = options;
  let offset = start;

  return new ReadableStream({
    async pull(controller) {
      if (offset > end) {
        controller.close();
        return;
      }
      
      const chunkEnd = Math.min(offset + chunkSize - 1, end);
      try {
        const chunk = await readChunk(offset, chunkEnd);
        controller.enqueue(chunk);
        offset = chunkEnd + 1;
      } catch (err) {
        controller.error(err);
      }
    },
  });
}

/**
 * Helper to create a streaming response for range requests.
 * 
 * @param {Object} options
 * @param {number} options.totalSize - Total file size
 * @param {string} [options.rangeHeader] - HTTP Range header value
 * @param {string} [options.contentType='application/octet-stream'] - MIME type
 * @param {function(start: number, end: number): Promise<Uint8Array>} options.readChunk
 * @returns {{ stream: ReadableStream, headers: Headers, status: number }}
 * 
 * @example
 * // In a Service Worker
 * const { stream, headers, status } = createRangeResponse({
 *   totalSize: innerFile.length,
 *   rangeHeader: request.headers.get('Range'),
 *   contentType: 'video/mp4',
 *   readChunk: async (start, end) => decompress(start, end),
 * });
 * return new Response(stream, { status, headers });
 */
export function createRangeResponse(options) {
  const { totalSize, rangeHeader, contentType = 'application/octet-stream', readChunk } = options;
  
  let start = 0;
  let end = totalSize - 1;
  let status = 200;
  
  const headers = new Headers({
    'Content-Type': contentType,
    'Accept-Ranges': 'bytes',
  });
  
  // Parse Range header if present
  if (rangeHeader) {
    const match = rangeHeader.match(/bytes=(\d*)-(\d*)/);
    if (match) {
      start = match[1] ? parseInt(match[1], 10) : 0;
      end = match[2] ? parseInt(match[2], 10) : totalSize - 1;
      status = 206;
      headers.set('Content-Range', `bytes ${start}-${end}/${totalSize}`);
    }
  }
  
  headers.set('Content-Length', String(end - start + 1));
  
  const stream = createReadableStream({
    totalSize,
    start,
    end,
    readChunk,
  });
  
  return { stream, headers, status };
}
