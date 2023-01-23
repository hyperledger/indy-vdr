import { PoolCreate } from '@hyperledger/indy-vdr-shared'

import { SOVRIN_GENESIS_TRANSACTION_BUILDER_NET } from './fixtures'

export const setupPool = () => {
  return new PoolCreate({ parameters: { transactions: SOVRIN_GENESIS_TRANSACTION_BUILDER_NET } })
}
