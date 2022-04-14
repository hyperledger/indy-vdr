import {
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
  GetRichSchemaObjectByIdRequestOptions,
  GetRichSchemaObjectByMetadataRequestOptions,
  GetSchemaRequestOptions,
  GetTransactionAuthorAgreementRequestOptions,
  GetTransactionRequestOptions,
  GetValidatorInfoRequestOptions,
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
  RichSchemaRequestOptions,
  SchemaRequestOptions,
  TransactionAuthorAgreementRequestOptions,
  Transactions,
  Verifiers,
} from 'indy-vdr-shared'
import { indyVdrReactNative } from '../register'
import { serializeArguments, SerializedOptions } from '../utils'

export class ReactNativeIndyVdr implements IndyVdr {
  private promisify = (method: (cb: (...args: any[]) => void) => void): Promise<void> => {
    return new Promise((resolve, reject) => {
      const _cb = (err: number) => {
        if (err !== 0) reject(this.getCurrentError())
        resolve()
      }

      method(_cb)
    })
  }

  private promisifyWithResponse = <T>(method: (cb: (...args: any[]) => void) => void, isStream = false): Promise<T> => {
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

  getCurrentError(): string {
    return indyVdrReactNative.getCurrentError({})
  }

  version(): string {
    return indyVdrReactNative.version({})
  }

  setConfig(options: { config: Record<string, unknown> }): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.setConfig(serializedOptions)
  }

  setDefaultLogger(): void {
    indyVdrReactNative.setDefaultLogger({})
  }

  setProtocolVersion(options: { version: number }): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.setProtocolVersion(serializedOptions)
  }

  setSocksProxy(options: { socksProxy: string }): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.setSocksProxy(serializedOptions)
  }

  buildAcceptanceMechanismsRequest(options: AcceptanceMechanismsRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildAcceptanceMechanismsRequest(serializedOptions)
  }

  buildGetAcceptanceMechanismsRequest(options: GetAcceptanceMechanismsRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetAcceptanceMechanismsRequest(serializedOptions)
  }

  buildAttribRequest(options: AttribRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildAttribRequest(serializedOptions)
  }

  buildGetAttribRequest(options: GetAttribRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetAttribRequest(serializedOptions)
  }

  buildCredDefRequest(options: CredentialDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildCredDefRequest(serializedOptions)
  }

  buildGetCredDefRequest(options: GetCredentialDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetCredDefRequest(serializedOptions)
  }

  buildGetRevocRegDefRequest(options: GetRevocationRegistryDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetRevocRegDefRequest(serializedOptions)
  }

  buildGetRevocRegRequest(options: GetRevocationRegistryRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetRevocRegRequest(serializedOptions)
  }

  buildGetRevocRegDeltaRequest(options: GetRevocationRegistryDeltaRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetRevocRegDeltaRequest(serializedOptions)
  }

  buildRevocRegDefRequest(options: RevocationRegistryDefinitionRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildRevocRegDefRequest(serializedOptions)
  }

  buildCustomRequest(options: CustomRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildCustomRequest(serializedOptions)
  }

  buildDisableAllTxnAuthorAgreementsRequest(options: DisableAllTransactionAuthorAgreementsRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildDisableAllTxnAuthorAgreementsRequest(serializedOptions)
  }

  buildGetNymRequest(options: GetNymRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetNymRequest(serializedOptions)
  }

  buildGetSchemaRequest(options: GetSchemaRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetSchemaRequest(serializedOptions)
  }

  buildGetTxnAuthorAgreementRequest(options: GetTransactionAuthorAgreementRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetTxnAuthorAgreementRequest(serializedOptions)
  }

  buildGetTxnRequest(options: GetTransactionRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetTxnRequest(serializedOptions)
  }

  buildGetValidatorInfoRequest(options: GetValidatorInfoRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetValidatorInfoRequest(serializedOptions)
  }

  buildNymRequest(options: NymRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildNymRequest(serializedOptions)
  }

  buildRevocRegEntryRequest(options: RevocationRegistryEntryRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildRevocRegEntryRequest(serializedOptions)
  }

  buildSchemaRequest(options: SchemaRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildSchemaRequest(serializedOptions)
  }

  buildTxnAuthorAgreementRequest(options: TransactionAuthorAgreementRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildTxnAuthorAgreementRequest(serializedOptions)
  }

  buildRichSchemaRequest(options: RichSchemaRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildRichSchemaRequest(serializedOptions)
  }

  buildGetRichSchemaObjectByIdRequest(options: GetRichSchemaObjectByIdRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetRichSchemaObjectByIdRequest(serializedOptions)
  }

  buildGetRichSchemaObjectByMetadataRequest(options: GetRichSchemaObjectByMetadataRequestOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.buildGetRichSchemaObjectByMetadataRequest(serializedOptions)
  }

  poolCreate(options: PoolCreateOptions): number {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.poolCreate(serializedOptions)
  }

  async poolRefresh(options: { poolHandle: PoolHandle }): Promise<void> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>
    return this.promisify((cb) => indyVdrReactNative.poolRefresh({ cb, poolHandle }))
  }

  poolGetStatus(options: { poolHandle: PoolHandle }): Promise<PoolStatus> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>
    return this.promisifyWithResponse<PoolStatus>((cb) => indyVdrReactNative.poolGetStatus({ cb, poolHandle }))
  }

  poolGetTransactions(options: { poolHandle: PoolHandle }): Promise<Transactions> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>
    return this.promisifyWithResponse<Transactions>((cb) => indyVdrReactNative.poolGetTransactions({ cb, poolHandle }))
  }

  poolGetVerifiers(options: { poolHandle: PoolHandle }): Promise<Verifiers> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>
    return this.promisifyWithResponse<Verifiers>((cb) => indyVdrReactNative.poolGetVerifiers({ cb, poolHandle }))
  }

  poolSubmitAction(options: PoolSubmitActionOptions & { poolHandle: PoolHandle }): Promise<string> {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return this.promisifyWithResponse<string>((cb) => indyVdrReactNative.poolSubmitAction({ cb, ...serializedOptions }))
  }

  poolSubmitRequest(options: PoolSubmitRequestOptions & { poolHandle: PoolHandle }): Promise<string> {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return this.promisifyWithResponse((cb) => indyVdrReactNative.poolSubmitRequest({ cb, ...serializedOptions }))
  }

  poolClose(options: { poolHandle: number }): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.poolClose(serializedOptions)
  }

  prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.prepareTxnAuthorAgreementAcceptance(serializedOptions)
  }

  requestFree(options: { requestHandle: number }): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.requestFree(serializedOptions)
  }

  requestGetBody<T extends Record<string, unknown>>(options: { requestHandle: number }): T {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return JSON.parse(indyVdrReactNative.requestGetBody(serializedOptions)) as T
  }

  requestGetSignatureInput(options: { requestHandle: number }): string {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    return indyVdrReactNative.requestGetSignatureInput(serializedOptions)
  }

  requestSetEndorser(options: RequestSetEndorserOptions & { requestHandle: RequestHandle }): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.requestSetEndorser(serializedOptions)
  }

  requestSetMultiSignature(options: RequestSetMultiSignatureOptions & { requestHandle: RequestHandle }): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.requestSetMultiSignature(serializedOptions)
  }

  requestSetSignature(options: RequestSetSignatureOptions): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.requestSetSignature(serializedOptions)
  }

  requestSetTxnAuthorAgreementAcceptance(
    options: RequestSetTxnAuthorAgreementAcceptanceOptions & { requestHandle: RequestHandle }
  ): void {
    const serializedOptions = serializeArguments(options) as SerializedOptions<typeof options>
    indyVdrReactNative.requestSetTxnAuthorAgreementAcceptance(serializedOptions)
  }
}
