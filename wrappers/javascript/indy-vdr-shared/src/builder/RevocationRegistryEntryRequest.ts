import type { WriteRequestResultTxnBase, WriteRequestResponse } from '../types'

import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type RevocationRegistryEntryRequestOptions = {
  submitterDid: string
  revocationRegistryDefinitionId: string
  revocationRegistryDefinitionType: string
  revocationRegistryEntry: {
    ver: '1.0'
    value: { accum: string }
  }
}

interface RevocationRegistryEntryResultTxn extends WriteRequestResultTxnBase {
  type: '114'
  data: {
    value: {
      accum: string
    }
    revocRegDefId: string
    revocDefType: 'CL_ACCUM'
  }
}

export type RevocationRegistryEntryResponse = WriteRequestResponse<RevocationRegistryEntryResultTxn>

export class RevocationRegistryEntryRequest extends IndyVdrRequest<RevocationRegistryEntryResponse> {
  public constructor(options: RevocationRegistryEntryRequestOptions) {
    const handle = indyVdr.buildRevocRegEntryRequest(options)
    super({ handle })
  }
}
