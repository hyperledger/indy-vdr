import type { Config } from '@jest/types'

const config: Config.InitialOptions = {
  verbose: true,
  testTimeout: 120000,
  moduleNameMapper: {
    '^indy-vdr-shared$': '<rootDir>/../shared/src',
    '^indy-vdr-nodejs$': '<rootDir>/src',
  },
}

export default config
