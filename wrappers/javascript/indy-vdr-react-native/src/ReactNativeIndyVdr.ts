import type {
  AcceptanceMechanismsRequestOptions,
  AttribRequestOptions,
  CredentialDefinitionRequestOptions,
  CustomRequestOptions,
  DisableAllTransactionAuthorAgreementsRequestOptions,
  GetAcceptanceMechanismsRequestOptions,
  GetAttribRequestOptions,
  GetCredentialDefinitionRequestOptions,
  GetNymRequestOptions,
  GetRevocationRegistryDefinitionRequestOptions,
  GetRevocationRegistryDeltaRequestOptions,
  GetRevocationRegistryRequestOptions,
  GetSchemaRequestOptions,
  GetTransactionAuthorAgreementRequestOptions,
  GetTransactionRequestOptions,
  GetValidatorInfoActionOptions,
  IndyVdr,
  NymRequestOptions,
  PoolCreateOptions,
  PoolHandle,
  PoolStatus,
  PoolSubmitActionOptions,
  PoolSubmitRequestOptions,
  PrepareTxnAuthorAgreementAcceptanceOptions,
  RequestHandle,
  RequestSetEndorserOptions,
  RequestSetMultiSignatureOptions,
  RequestSetSignatureOptions,
  RequestSetTxnAuthorAgreementAcceptanceOptions,
  RevocationRegistryDefinitionRequestOptions,
  RevocationRegistryEntryRequestOptions,
  SchemaRequestOptions,
  TransactionAuthorAgreementRequestOptions,
  Transactions,
  Verifiers,
} from '@hyperledger/indy-vdr-shared'

import { indyVdrReactNative } from './library'
import { serializeArguments } from './serialize'

export class ReactNativeIndyVdr implements IndyVdr {
  private promisify = (method: (cb: (err: number) => void) => void): Promise<void> => {
    return new Promise((resolve, reject) => {
      const _cb = (err: number) => {
        if (err !== 0) reject(this.getCurrentError())
        resolve()
      }

      method(_cb)
    })
  }

  private promisifyWithResponse = <T>(
    method: (cb: (err: number, response: string) => void) => void,
    isStream = false
  ): Promise<T> => {
    return new Promise((resolve, reject) => {
      const _cb = (err: number, response: string) => {
        if (err !== 0) reject(this.getCurrentError())

        try {
          // this is required to add array brackets, and commas, to an invalid json object that
          //should be a list
          const mappedResponse = isStream ? '[' + response.replace(/\n/g, ',') + ']' : response
          resolve(JSON.parse(mappedResponse) as T)
        } catch (error) {
          resolve(JSON.parse(response) as T)
        }
      }

      method(_cb)
    })
  }

  public getCurrentError(): string {
    return indyVdrReactNative.getCurrentError({})
  }

  public version(): string {
    return indyVdrReactNative.version({})
  }

  public setConfig(options: { config: Record<string, unknown> }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.setConfig(serializedOptions)
  }

  public setDefaultLogger(): void {
    indyVdrReactNative.setDefaultLogger({})
  }

  public setProtocolVersion(options: { version: number }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.setProtocolVersion(serializedOptions)
  }

  public setSocksProxy(options: { socksProxy: string }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.setSocksProxy(serializedOptions)
  }

  public buildAcceptanceMechanismsRequest(options: AcceptanceMechanismsRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildAcceptanceMechanismsRequest(serializedOptions)
  }

  public buildGetAcceptanceMechanismsRequest(options: GetAcceptanceMechanismsRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetAcceptanceMechanismsRequest(serializedOptions)
  }

  public buildAttribRequest(options: AttribRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildAttribRequest(serializedOptions)
  }

  public buildGetAttribRequest(options: GetAttribRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetAttribRequest(serializedOptions)
  }

  public buildCredDefRequest(options: CredentialDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildCredDefRequest(serializedOptions)
  }

  public buildGetCredDefRequest(options: GetCredentialDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetCredDefRequest(serializedOptions)
  }

  public buildGetRevocRegDefRequest(options: GetRevocationRegistryDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetRevocRegDefRequest(serializedOptions)
  }

  public buildGetRevocRegRequest(options: GetRevocationRegistryRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetRevocRegRequest(serializedOptions)
  }

  public buildGetRevocRegDeltaRequest(options: GetRevocationRegistryDeltaRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetRevocRegDeltaRequest(serializedOptions)
  }

  public buildRevocRegDefRequest(options: RevocationRegistryDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildRevocRegDefRequest(serializedOptions)
  }

  public buildCustomRequest(options: CustomRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildCustomRequest(serializedOptions)
  }

  public buildDisableAllTxnAuthorAgreementsRequest(
    options: DisableAllTransactionAuthorAgreementsRequestOptions
  ): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildDisableAllTxnAuthorAgreementsRequest(serializedOptions)
  }

  public buildGetNymRequest(options: GetNymRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetNymRequest(serializedOptions)
  }

  public buildGetSchemaRequest(options: GetSchemaRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetSchemaRequest(serializedOptions)
  }

  public buildGetTxnAuthorAgreementRequest(options: GetTransactionAuthorAgreementRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetTxnAuthorAgreementRequest(serializedOptions)
  }

  public buildGetTxnRequest(options: GetTransactionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetTxnRequest(serializedOptions)
  }

  public buildGetValidatorInfoRequest(options: GetValidatorInfoActionOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildGetValidatorInfoRequest(serializedOptions)
  }

  public buildNymRequest(options: NymRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildNymRequest(serializedOptions)
  }

  public buildRevocRegEntryRequest(options: RevocationRegistryEntryRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildRevocRegEntryRequest(serializedOptions)
  }

  public buildSchemaRequest(options: SchemaRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildSchemaRequest(serializedOptions)
  }

  public buildTxnAuthorAgreementRequest(options: TransactionAuthorAgreementRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.buildTxnAuthorAgreementRequest(serializedOptions)
  }

  public poolCreate(options: PoolCreateOptions): number {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.poolCreate(serializedOptions)
  }

  public async poolRefresh(options: { poolHandle: PoolHandle }): Promise<void> {
    const { poolHandle } = serializeArguments(options)
    return this.promisify((cb) => indyVdrReactNative.poolRefresh({ cb, poolHandle }))
  }

  public async poolGetStatus(options: { poolHandle: PoolHandle }): Promise<PoolStatus> {
    const { poolHandle } = serializeArguments(options)
    return this.promisifyWithResponse<PoolStatus>((cb) => indyVdrReactNative.poolGetStatus({ cb, poolHandle }))
  }

  public async poolGetTransactions(options: { poolHandle: PoolHandle }): Promise<Transactions> {
    const { poolHandle } = serializeArguments(options)
    return this.promisifyWithResponse<Transactions>(
      (cb) => indyVdrReactNative.poolGetTransactions({ cb, poolHandle }),
      true
    )
  }

  public async poolGetVerifiers(options: { poolHandle: PoolHandle }): Promise<Verifiers> {
    const { poolHandle } = serializeArguments(options)
    return this.promisifyWithResponse<Verifiers>((cb) => indyVdrReactNative.poolGetVerifiers({ cb, poolHandle }))
  }

  public async poolSubmitAction<T extends Record<string, unknown>>(
    options: PoolSubmitActionOptions & { poolHandle: PoolHandle }
  ): Promise<T> {
    const serializedOptions = serializeArguments(options)
    return this.promisifyWithResponse<T>((cb) => indyVdrReactNative.poolSubmitAction({ cb, ...serializedOptions }))
  }

  public async poolSubmitRequest<T extends Record<string, unknown>>(
    options: PoolSubmitRequestOptions & { poolHandle: PoolHandle }
  ): Promise<T> {
    const serializedOptions = serializeArguments(options)
    return this.promisifyWithResponse<T>((cb) => indyVdrReactNative.poolSubmitRequest({ cb, ...serializedOptions }))
  }

  public poolClose(options: { poolHandle: number }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.poolClose(serializedOptions)
  }

  public prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.prepareTxnAuthorAgreementAcceptance(serializedOptions)
  }

  public requestFree(options: { requestHandle: number }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.requestFree(serializedOptions)
  }

  public requestGetBody<T extends Record<string, unknown>>(options: { requestHandle: number }): T {
    const serializedOptions = serializeArguments(options)
    return JSON.parse(indyVdrReactNative.requestGetBody(serializedOptions)) as T
  }

  public requestGetSignatureInput(options: { requestHandle: number }): string {
    const serializedOptions = serializeArguments(options)
    return indyVdrReactNative.requestGetSignatureInput(serializedOptions)
  }

  public requestSetEndorser(options: RequestSetEndorserOptions & { requestHandle: RequestHandle }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.requestSetEndorser(serializedOptions)
  }

  public requestSetMultiSignature(options: RequestSetMultiSignatureOptions & { requestHandle: RequestHandle }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.requestSetMultiSignature(serializedOptions)
  }

  public requestSetSignature(options: RequestSetSignatureOptions & { requestHandle: RequestHandle }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.requestSetSignature(serializedOptions)
  }

  public requestSetTxnAuthorAgreementAcceptance(
    options: RequestSetTxnAuthorAgreementAcceptanceOptions & { requestHandle: RequestHandle }
  ): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.requestSetTxnAuthorAgreementAcceptance(serializedOptions)
  }
}
