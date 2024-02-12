import { defineConfig } from 'tsup'

export default defineConfig({
    entry: ['src/index.ts'],
    format: ['esm', 'cjs'],
    splitting: true,
    sourcemap: true,
    clean: true,
    dts: true,
})
