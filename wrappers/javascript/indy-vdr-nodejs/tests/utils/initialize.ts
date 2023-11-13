import { PoolCreate } from '@hyperledger/indy-vdr-shared'

import { genesisTxnPath } from './fixtures'

export const setupPool = () => {
  return new PoolCreate({ parameters: { transactions_path: genesisTxnPath } })
}
