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
  GetRichSchemaObjectByIdRequestOptions,
  GetRichSchemaObjectByMetadataRequestOptions,
  GetSchemaRequestOptions,
  GetTransactionAuthorAgreementRequestOptions,
  GetTransactionRequestOptions,
  GetValidatorInfoRequestOptions,
  NymRequestOptions,
  PoolCreateOptions,
  RevocationRegistryDefinitionRequestOptions,
  RevocationRegistryEntryRequestOptions,
  RichSchemaRequestOptions,
  SchemaRequestOptions,
  TransactionAuthorAgreementRequestOptions,
} from '../builder'
import type {
  PoolHandle,
  PoolSubmitActionOptions,
  PoolSubmitRequestOptions,
  RequestHandle,
  RequestSetEndorserOptions,
  RequestSetMultiSignatureOptions,
  RequestSetTxnAuthorAgreementAcceptanceOptions,
} from '../indyVdr'
import type { PrepareTxnAuthorAgreementAcceptanceOptions, RequestSetSignatureOptions } from './builderTypes'
import type { PoolStatus, Transactions, Verifiers } from './types'

// TODO: proper documentation
export interface IndyVdr {
  getCurrentError(): string

  version(): string

  setConfig(options: { config: Record<string, unknown> }): void

  setDefaultLogger(): void

  setProtocolVersion(options: { version: number }): void

  setSocksProxy(options: { socksProxy: string }): void

  buildAcceptanceMechanismsRequest(options: AcceptanceMechanismsRequestOptions): number

  buildGetAcceptanceMechanismsRequest(options: GetAcceptanceMechanismsRequestOptions): number

  buildAttribRequest(options: AttribRequestOptions): number

  buildGetAttribRequest(options: GetAttribRequestOptions): number

  buildCredDefRequest(options: CredentialDefinitionRequestOptions): number

  buildGetCredDefRequest(options: GetCredentialDefinitionRequestOptions): number

  buildGetRevocRegDefRequest(options: GetRevocationRegistryDefinitionRequestOptions): number

  buildGetRevocRegRequest(options: GetRevocationRegistryRequestOptions): number

  buildGetRevocRegDeltaRequest(options: GetRevocationRegistryDeltaRequestOptions): number

  buildRevocRegDefRequest(options: RevocationRegistryDefinitionRequestOptions): number

  buildCustomRequest(options: CustomRequestOptions): number

  buildDisableAllTxnAuthorAgreementsRequest(options: DisableAllTransactionAuthorAgreementsRequestOptions): number

  buildGetNymRequest(options: GetNymRequestOptions): number

  buildGetSchemaRequest(options: GetSchemaRequestOptions): number

  buildGetTxnAuthorAgreementRequest(options: GetTransactionAuthorAgreementRequestOptions): number

  buildGetTxnRequest(options: GetTransactionRequestOptions): number

  buildGetValidatorInfoRequest(options: GetValidatorInfoRequestOptions): number

  buildNymRequest(options: NymRequestOptions): number

  buildRevocRegEntryRequest(options: RevocationRegistryEntryRequestOptions): number

  buildSchemaRequest(options: SchemaRequestOptions): number

  buildTxnAuthorAgreementRequest(options: TransactionAuthorAgreementRequestOptions): number

  buildRichSchemaRequest(options: RichSchemaRequestOptions): number

  buildGetRichSchemaObjectByIdRequest(options: GetRichSchemaObjectByIdRequestOptions): number

  buildGetRichSchemaObjectByMetadataRequest(options: GetRichSchemaObjectByMetadataRequestOptions): number

  poolCreate(options: PoolCreateOptions): number

  poolRefresh(options: { poolHandle: PoolHandle }): Promise<void>

  poolGetStatus(options: { poolHandle: PoolHandle }): Promise<PoolStatus>

  poolGetTransactions(options: { poolHandle: PoolHandle }): Promise<Transactions>

  poolGetVerifiers(options: { poolHandle: PoolHandle }): Promise<Verifiers>

  poolSubmitAction(options: PoolSubmitActionOptions & { poolHandle: PoolHandle }): Promise<SubmitAction>

  poolSubmitRequest(options: PoolSubmitRequestOptions & { poolHandle: PoolHandle }): Promise<SubmitRequest>

  poolClose(options: { poolHandle: number }): void

  prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string

  requestFree(options: { requestHandle: number }): void

  requestGetBody<T extends Record<string, unknown>>(options: { requestHandle: number }): T

  requestGetSignatureInput(options: { requestHandle: number }): string

  requestSetEndorser(options: RequestSetEndorserOptions & { requestHandle: RequestHandle }): void

  requestSetMultiSignature(options: RequestSetMultiSignatureOptions & { requestHandle: RequestHandle }): void

  requestSetSignature(options: RequestSetSignatureOptions & { requestHandle: RequestHandle }): void

  requestSetTxnAuthorAgreementAcceptance(
    options: RequestSetTxnAuthorAgreementAcceptanceOptions & { requestHandle: RequestHandle }
  ): void
}
