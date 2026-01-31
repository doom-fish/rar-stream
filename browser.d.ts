/**
 * rar-stream browser type definitions
 */

export { InitInput, InitOutput, SyncInitInput } from './pkg/rar_stream.d.ts';

/** Initialize WASM module (must be called before using other functions) */
export { default as init } from './pkg/rar_stream.js';

/** Initialize WASM module synchronously */
export { initSync } from './pkg/rar_stream.js';

/** Check if data is a RAR archive */
export function isRarArchive(data: Uint8Array): boolean;

/** Get RAR version (15 for RAR4, 50 for RAR5, 0 if not RAR) */
export function getRarVersion(data: Uint8Array): number;

/** Parse RAR header information */
export function parseRarHeader(data: Uint8Array): {
  version: number;
  isMultiVolume: boolean;
  hasRecovery: boolean;
  isLocked: boolean;
  isSolid: boolean;
  hasAuthInfo: boolean;
};

/** RAR decompressor class */
export class RarDecoder {
  constructor(unpacked_size: bigint);
  free(): void;
  [Symbol.dispose](): void;
  bytes_written(): bigint;
  decompress(data: Uint8Array): Uint8Array;
  is_complete(): boolean;
  reset(): void;
}

// Snake_case aliases for compatibility
export { isRarArchive as is_rar_archive };
export { getRarVersion as get_rar_version };
export { parseRarHeader as parse_rar_header };
export { RarDecoder as WasmRarDecoder };
