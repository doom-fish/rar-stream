/**
 * rar-stream browser entry point with Web Streams API support
 * 
 * Provides ReadableStream support for streaming file content in browsers.
 */

// Re-export all WASM bindings
export {
  default as init,
  initSync,
  is_rar_archive as isRarArchive,
  get_rar_version as getRarVersion,
  parse_rar_header as parseRarHeader,
  WasmRarDecoder as RarDecoder,
} from '../pkg/rar_stream.js';

// Also export snake_case versions for compatibility
export {
  is_rar_archive,
  get_rar_version,
  parse_rar_header,
  WasmRarDecoder,
} from '../pkg/rar_stream.js';

// Re-export crypto if available
export { WasmRar5Crypto } from '../pkg/rar_stream.js';

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
