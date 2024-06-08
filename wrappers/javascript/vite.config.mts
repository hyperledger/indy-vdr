import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    testTimeout: 120000,
    alias: {
      '@hyperledger/indy-vdr-nodejs': new URL(
        './indy-vdr-nodejs/src',
        import.meta.url,
      ).pathname,
      '@hyperledger/indy-vdr-shared': new URL(
        './indy-vdr-shared/src',
        import.meta.url,
      ).pathname,
    },
  },
})
