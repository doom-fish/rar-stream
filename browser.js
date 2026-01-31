/**
 * rar-stream browser entry point
 * Re-exports WASM bindings with unified API
 */

// Re-export all WASM bindings
export {
  default as init,
  initSync,
  is_rar_archive as isRarArchive,
  get_rar_version as getRarVersion,
  parse_rar_header as parseRarHeader,
  WasmRarDecoder as RarDecoder,
} from './pkg/rar_stream.js';

// Also export snake_case versions for compatibility
export {
  is_rar_archive,
  get_rar_version,
  parse_rar_header,
  WasmRarDecoder,
} from './pkg/rar_stream.js';
