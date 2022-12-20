/* eslint-disable no-empty-pattern */

export type PoolHandle = number
export type RequestHandle = number

type Callback = (err: number) => void

type CallbackWithResponse = (err: number, response: string) => void

export interface IndyVdrNativeBindings {
  version({}): string

  getCurrentError({}): string

  setConfig(options: { config: string }): null

  setDefaultLogger({}): null

  setProtocolVersion(options: { version: number }): null

  setSocksProxy(options: { socksProxy: string }): null

  buildAcceptanceMechanismsRequest(options: {
    submitterDid: string
    aml: string
    version: string
    amlContext?: string
  }): RequestHandle

  buildGetAcceptanceMechanismsRequest(options: {
    submitterDid?: string
    timestamp?: number
    version?: string
  }): RequestHandle

  buildAttribRequest(options: {
    submitterDid: string
    targetDid: string
    hash?: string
    raw?: string
    enc?: string
  }): RequestHandle

  buildGetAttribRequest(options: {
    submitterDid?: string
    targetDid: string
    raw?: string
    hash?: string
    enc?: string
  }): RequestHandle

  buildCredDefRequest(options: { submitterDid: string; credentialDefinition: string }): RequestHandle

  buildGetCredDefRequest(options: { submitterDid?: string; credentialDefinitionId: string }): RequestHandle

  buildGetRevocRegDefRequest(options: { submitterDid?: string; revocationRegistryId: string }): RequestHandle

  buildGetRevocRegRequest(options: {
    submitterDid?: string
    revocationRegistryId: string
    timestamp: number
  }): RequestHandle

  buildGetRevocRegDeltaRequest(options: {
    submitterDid?: string
    revocationRegistryId: string
    fromTs?: number
    toTs: number
  }): RequestHandle

  buildRevocRegDefRequest(options: { submitterDid: string; revocationRegistryDefinitionV1: string }): RequestHandle

  buildCustomRequest(options: { customRequest: string }): RequestHandle

  buildDisableAllTxnAuthorAgreementsRequest(options: { submitterDid: string }): RequestHandle

  buildGetNymRequest(options: { submitterDid?: string; dest: string }): RequestHandle

  buildGetSchemaRequest(options: { submitterDid?: string; schemaId: string }): RequestHandle

  buildGetTxnAuthorAgreementRequest(options: { submitterDid?: string; data?: string }): RequestHandle

  buildGetTxnRequest(options: { submitterDid?: string; ledgerType: number; seqNo: number }): RequestHandle

  buildGetValidatorInfoRequest(options: { submitterDid: string }): RequestHandle

  buildNymRequest(options: {
    submitterDid: string
    dest: string
    verkey?: string
    alias?: string
    role?: string
  }): RequestHandle

  buildRevocRegEntryRequest(options: {
    submitterDid: string
    revocationRegistryDefinitionId: string
    revocationRegistryDefinitionType: string
    revocationRegistryEntry: string
  }): RequestHandle

  buildSchemaRequest(options: { submitterDid: string; schema: string }): RequestHandle

  buildTxnAuthorAgreementRequest(options: {
    submitterDid: string
    text?: string
    version: string
    ratificationTs?: number
    retirementTs?: number
  }): RequestHandle

  poolCreate(options: { parameters: string }): PoolHandle

  poolRefresh(options: { poolHandle: PoolHandle; cb: Callback }): null

  poolGetStatus(options: { poolHandle: PoolHandle; cb: CallbackWithResponse }): null

  poolGetTransactions(options: { poolHandle: PoolHandle; cb: CallbackWithResponse }): null

  poolGetVerifiers(options: { poolHandle: PoolHandle; cb: CallbackWithResponse }): null

  poolSubmitAction(options: {
    poolHandle: PoolHandle
    requestHandle: number
    nodes?: string
    timeout?: number
    cb: CallbackWithResponse
  }): null

  poolSubmitRequest(options: { poolHandle: PoolHandle; requestHandle: number; cb: CallbackWithResponse }): null

  poolClose(options: { poolHandle: PoolHandle }): null

  prepareTxnAuthorAgreementAcceptance(options: {
    text?: string
    version?: string
    taaDigest?: string
    acceptanceMechanismType: string
    time: number
  }): string

  requestFree(options: { requestHandle: number }): null

  requestGetBody(options: { requestHandle: number }): string

  requestGetSignatureInput(options: { requestHandle: number }): string

  requestSetEndorser(options: { requestHandle: number; endorser: string }): null

  requestSetMultiSignature(options: { requestHandle: number; identifier: string; signature: ArrayBuffer }): null

  requestSetSignature(options: { requestHandle: number; signature: ArrayBuffer }): null

  requestSetTxnAuthorAgreementAcceptance(options: { requestHandle: number; acceptance: string }): null
}
