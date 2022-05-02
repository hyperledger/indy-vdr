import { PoolCreate, registerIndyVdr } from 'indy-vdr-shared'

import { indyVdrNodeJS } from '../../src'

import { SOVRIN_GENESIS_TRANSACTION_BUILDER_NET } from './fixtures'

export const setupPool = () => {
  registerIndyVdr({ vdr: indyVdrNodeJS })
  return new PoolCreate({ parameters: { transactions: SOVRIN_GENESIS_TRANSACTION_BUILDER_NET } })
}
