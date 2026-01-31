// Test configuration for rar-stream (Rust NAPI module)
import { defineConfig } from 'vitest/config'
import path from 'path'

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['**/*.test.ts'],
    exclude: ['test-browser/**', 'node_modules/**', 'target/**'],
    testTimeout: 30000,
  },
  resolve: {
    alias: {
      'rar-stream': path.resolve(__dirname, './index.js'),
    },
  },
})
