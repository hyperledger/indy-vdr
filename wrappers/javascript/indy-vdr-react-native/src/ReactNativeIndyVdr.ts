import type { Callback, CallbackWithResponse } from './utils'
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
  IndyVdrErrorObject,
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

import { handleInvalidNullResponse, IndyVdrError } from '@hyperledger/indy-vdr-shared'

import { indyVdrReactNative } from './library'
import { handleError, serializeArguments } from './utils'

export class ReactNativeIndyVdr implements IndyVdr {
  private promisify = (method: (cb: Callback) => void): Promise<void> => {
    return new Promise((resolve, reject) => {
      const _cb: Callback = ({ errorCode }) => {
        if (errorCode !== 0) reject(this.getCurrentError())
        resolve()
      }

      method(_cb)
    })
  }

  private promisifyWithResponse = <Return>(
    method: (cb: CallbackWithResponse<Return>) => void,
    isStream = false
  ): Promise<Return | null> => {
    return new Promise((resolve, reject) => {
      const _cb: CallbackWithResponse = ({ value, errorCode }) => {
        if (errorCode !== 0) reject(new IndyVdrError(JSON.parse(this.getCurrentError()) as IndyVdrErrorObject))

        // this is required to add array brackets, and commas, to an invalid json object that
        //should be a list
        if (typeof value === 'string' && isStream) {
          const mappedResponse = isStream ? '[' + value.replace(/\n/g, ',') + ']' : value

          if (mappedResponse.length === 0) return resolve(null)
          resolve(JSON.parse(mappedResponse) as Return)
        } else {
          resolve(value as Return)
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
    handleError(indyVdrReactNative.setDefaultLogger({}))
  }

  public setProtocolVersion(options: { version: number }): void {
    const serializedOptions = serializeArguments(options)
    handleError(indyVdrReactNative.setProtocolVersion(serializedOptions))
  }

  public setSocksProxy(options: { socksProxy: string }): void {
    const serializedOptions = serializeArguments(options)
    handleError(indyVdrReactNative.setSocksProxy(serializedOptions))
  }

  public buildAcceptanceMechanismsRequest(options: AcceptanceMechanismsRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(
      handleError(indyVdrReactNative.buildAcceptanceMechanismsRequest(serializedOptions))
    )
  }

  public buildGetAcceptanceMechanismsRequest(options: GetAcceptanceMechanismsRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(
      handleError(indyVdrReactNative.buildGetAcceptanceMechanismsRequest(serializedOptions))
    )
  }

  public buildAttribRequest(options: AttribRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildAttribRequest(serializedOptions)))
  }

  public buildGetAttribRequest(options: GetAttribRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetAttribRequest(serializedOptions)))
  }

  public buildCredDefRequest(options: CredentialDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildCredDefRequest(serializedOptions)))
  }

  public buildGetCredDefRequest(options: GetCredentialDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetCredDefRequest(serializedOptions)))
  }

  public buildGetRevocRegDefRequest(options: GetRevocationRegistryDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetRevocRegDefRequest(serializedOptions)))
  }

  public buildGetRevocRegRequest(options: GetRevocationRegistryRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetRevocRegRequest(serializedOptions)))
  }

  public buildGetRevocRegDeltaRequest(options: GetRevocationRegistryDeltaRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetRevocRegDeltaRequest(serializedOptions)))
  }

  public buildRevocRegDefRequest(options: RevocationRegistryDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildRevocRegDefRequest(serializedOptions)))
  }

  public buildCustomRequest(options: CustomRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildCustomRequest(serializedOptions)))
  }

  public buildDisableAllTxnAuthorAgreementsRequest(
    options: DisableAllTransactionAuthorAgreementsRequestOptions
  ): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(
      handleError(indyVdrReactNative.buildDisableAllTxnAuthorAgreementsRequest(serializedOptions))
    )
  }

  public buildGetNymRequest(options: GetNymRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetNymRequest(serializedOptions)))
  }

  public buildGetSchemaRequest(options: GetSchemaRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetSchemaRequest(serializedOptions)))
  }

  public buildGetTxnAuthorAgreementRequest(options: GetTransactionAuthorAgreementRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(
      handleError(indyVdrReactNative.buildGetTxnAuthorAgreementRequest(serializedOptions))
    )
  }

  public buildGetTxnRequest(options: GetTransactionRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetTxnRequest(serializedOptions)))
  }

  public buildGetValidatorInfoRequest(options: GetValidatorInfoActionOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildGetValidatorInfoRequest(serializedOptions)))
  }

  public buildNymRequest(options: NymRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildNymRequest(serializedOptions)))
  }

  public buildRevocRegEntryRequest(options: RevocationRegistryEntryRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildRevocRegEntryRequest(serializedOptions)))
  }

  public buildSchemaRequest(options: SchemaRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildSchemaRequest(serializedOptions)))
  }

  public buildTxnAuthorAgreementRequest(options: TransactionAuthorAgreementRequestOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.buildTxnAuthorAgreementRequest(serializedOptions)))
  }

  public poolCreate(options: PoolCreateOptions): number {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.poolCreate(serializedOptions)))
  }

  public async poolRefresh(options: { poolHandle: PoolHandle }): Promise<void> {
    const { poolHandle } = serializeArguments(options)
    return this.promisify((cb) => handleError(indyVdrReactNative.poolRefresh({ cb, poolHandle })))
  }

  public async poolGetStatus(options: { poolHandle: PoolHandle }): Promise<PoolStatus> {
    const { poolHandle } = serializeArguments(options)
    const result = handleInvalidNullResponse(
      await this.promisifyWithResponse<string>((cb) =>
        handleError(indyVdrReactNative.poolGetStatus({ cb, poolHandle }))
      )
    )

    return JSON.parse(result) as PoolStatus
  }

  public async poolGetTransactions(options: { poolHandle: PoolHandle }): Promise<Transactions> {
    const { poolHandle } = serializeArguments(options)
    const result = handleInvalidNullResponse(
      await this.promisifyWithResponse<string>((cb) => indyVdrReactNative.poolGetTransactions({ cb, poolHandle }), true)
    )

    return JSON.parse(result) as Transactions
  }

  public async poolGetVerifiers(options: { poolHandle: PoolHandle }): Promise<Verifiers> {
    const { poolHandle } = serializeArguments(options)
    const result = handleInvalidNullResponse(
      await this.promisifyWithResponse<string>((cb) => indyVdrReactNative.poolGetVerifiers({ cb, poolHandle }))
    )

    return JSON.parse(result) as Verifiers
  }

  public async poolSubmitAction<T extends Record<string, unknown>>(
    options: PoolSubmitActionOptions & { poolHandle: PoolHandle }
  ): Promise<T> {
    const serializedOptions = serializeArguments(options)
    const result = handleInvalidNullResponse(
      await this.promisifyWithResponse<string>((cb) =>
        indyVdrReactNative.poolSubmitAction({ cb, ...serializedOptions })
      )
    )

    return JSON.parse(result) as T
  }

  public async poolSubmitRequest<T extends Record<string, unknown>>(
    options: PoolSubmitRequestOptions & { poolHandle: PoolHandle }
  ): Promise<T> {
    const serializedOptions = serializeArguments(options)
    const result = handleInvalidNullResponse(
      await this.promisifyWithResponse<string>((cb) =>
        indyVdrReactNative.poolSubmitRequest({ cb, ...serializedOptions })
      )
    )

    return JSON.parse(result) as T
  }

  public poolClose(options: { poolHandle: number }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.poolClose(serializedOptions)
  }

  public prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(
      handleError(indyVdrReactNative.prepareTxnAuthorAgreementAcceptance(serializedOptions))
    )
  }

  public requestFree(options: { requestHandle: number }): void {
    const serializedOptions = serializeArguments(options)
    indyVdrReactNative.requestFree(serializedOptions)
  }

  public requestGetBody<T extends Record<string, unknown>>(options: { requestHandle: number }): T {
    const serializedOptions = serializeArguments(options)
    const result = handleError(indyVdrReactNative.requestGetBody(serializedOptions))
    return JSON.parse(result) as T
  }

  public requestGetSignatureInput(options: { requestHandle: number }): string {
    const serializedOptions = serializeArguments(options)
    return handleInvalidNullResponse(handleError(indyVdrReactNative.requestGetSignatureInput(serializedOptions)))
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
