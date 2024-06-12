import tsconfigPathsPlugin from 'vite-tsconfig-paths'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [tsconfigPathsPlugin()],
  test: {
    testTimeout: 120000,
    reporters: ['verbose'],
  },
})
