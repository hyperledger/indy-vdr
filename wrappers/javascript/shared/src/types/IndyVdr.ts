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
  PoolSubmitActionOptions,
  PoolSubmitRequestOptions,
  RequestSetEndorserOptions,
  RequestSetMultiSignatureOptions,
  RequestSetTxnAuthorAgreementAcceptanceOptions,
} from '../indyVdr'
import type { PrepareTxnAuthorAgreementAcceptanceOptions, RequestSetSignatureOptions } from './builderTypes'
import type { PoolStatus, SubmitAction, SubmitRequest, Transactions, Verifiers } from './types'

// TODO: proper documentation
export interface IndyVdr {
  get latestError(): string

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

  poolRefresh(handle: number): Promise<void>

  poolGetStatus(handle: number): Promise<PoolStatus>

  poolGetTransactions(handle: number): Promise<Transactions>

  poolGetVerifiers(handle: number): Promise<Verifiers>

  poolSubmitAction(handle: number, options: PoolSubmitActionOptions): Promise<SubmitAction>

  poolSubmitRequest(handle: number, options: PoolSubmitRequestOptions): Promise<SubmitRequest>

  poolClose(handle: number): void

  prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string

  requestFree(handle: number): void

  requestGetBody<T extends Record<string, unknown>>(handle: number): T

  requestGetSignatureInput(handle: number): string

  requestSetEndorser(handle: number, options: RequestSetEndorserOptions): void

  requestSetMultiSignature(handle: number, options: RequestSetMultiSignatureOptions): void

  requestSetSignature(handle: number, options: RequestSetSignatureOptions): void

  requestSetTxnAuthorAgreementAcceptance(handle: number, options: RequestSetTxnAuthorAgreementAcceptanceOptions): void
}
