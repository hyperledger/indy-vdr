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
  IndyVdr,
  NymRequestOptions,
  PoolCreateOptions,
  PoolStatus,
  PoolSubmitActionOptions,
  PoolSubmitRequestOptions,
  PrepareTxnAuthorAgreementAcceptanceOptions,
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

export class NodeJSIndyVdr implements IndyVdr {
  public getCurrentError(): string {
    throw new Error('Method not implemented.')
  }
  public version(): string {
    throw new Error('Method not implemented.')
  }
  public setConfig(options: { config: Record<string, unknown> }): void {
    throw new Error('Method not implemented.')
  }
  public setDefaultLogger(): void {
    throw new Error('Method not implemented.')
  }
  public setProtocolVersion(options: { version: number }): void {
    throw new Error('Method not implemented.')
  }
  public setSocksProxy(options: { socksProxy: string }): void {
    throw new Error('Method not implemented.')
  }
  public buildAcceptanceMechanismsRequest(options: AcceptanceMechanismsRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetAcceptanceMechanismsRequest(options: GetAcceptanceMechanismsRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildAttribRequest(options: AttribRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetAttribRequest(options: GetAttribRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildCredDefRequest(options: CredentialDefinitionRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetCredDefRequest(options: GetCredentialDefinitionRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetRevocRegDefRequest(options: GetRevocationRegistryDefinitionRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetRevocRegRequest(options: GetRevocationRegistryRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetRevocRegDeltaRequest(options: GetRevocationRegistryDeltaRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildRevocRegDefRequest(options: RevocationRegistryDefinitionRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildCustomRequest(options: CustomRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildDisableAllTxnAuthorAgreementsRequest(
    options: DisableAllTransactionAuthorAgreementsRequestOptions
  ): number {
    throw new Error('Method not implemented.')
  }
  public buildGetNymRequest(options: GetNymRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetSchemaRequest(options: GetSchemaRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetTxnAuthorAgreementRequest(options: GetTransactionAuthorAgreementRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetTxnRequest(options: GetTransactionRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetValidatorInfoRequest(options: GetValidatorInfoRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildNymRequest(options: NymRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildRevocRegEntryRequest(options: RevocationRegistryEntryRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildSchemaRequest(options: SchemaRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildTxnAuthorAgreementRequest(options: TransactionAuthorAgreementRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildRichSchemaRequest(options: RichSchemaRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetRichSchemaObjectByIdRequest(options: GetRichSchemaObjectByIdRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public buildGetRichSchemaObjectByMetadataRequest(options: GetRichSchemaObjectByMetadataRequestOptions): number {
    throw new Error('Method not implemented.')
  }
  public poolCreate(options: PoolCreateOptions): number {
    throw new Error('Method not implemented.')
  }
  public poolRefresh(options: { poolHandle: number }): Promise<void> {
    throw new Error('Method not implemented.')
  }
  public poolGetStatus(options: { poolHandle: number }): Promise<PoolStatus> {
    throw new Error('Method not implemented.')
  }
  public poolGetTransactions(options: { poolHandle: number }): Promise<Transactions> {
    throw new Error('Method not implemented.')
  }
  public poolGetVerifiers(options: { poolHandle: number }): Promise<Verifiers> {
    throw new Error('Method not implemented.')
  }
  public poolSubmitAction(options: PoolSubmitActionOptions & { poolHandle: number }): Promise<string> {
    throw new Error('Method not implemented.')
  }
  public poolSubmitRequest(options: PoolSubmitRequestOptions & { poolHandle: number }): Promise<string> {
    throw new Error('Method not implemented.')
  }
  public poolClose(options: { poolHandle: number }): void {
    throw new Error('Method not implemented.')
  }
  prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string {
    throw new Error('Method not implemented.')
  }
  public requestFree(options: { requestHandle: number }): void {
    throw new Error('Method not implemented.')
  }
  public requestGetBody<T extends Record<string, unknown>>(options: { requestHandle: number }): T {
    throw new Error('Method not implemented.')
  }
  public requestGetSignatureInput(options: { requestHandle: number }): string {
    throw new Error('Method not implemented.')
  }
  public requestSetEndorser(options: RequestSetEndorserOptions & { requestHandle: number }): void {
    throw new Error('Method not implemented.')
  }
  public requestSetMultiSignature(options: RequestSetMultiSignatureOptions & { requestHandle: number }): void {
    throw new Error('Method not implemented.')
  }
  public requestSetSignature(options: RequestSetSignatureOptions & { requestHandle: number }): void {
    throw new Error('Method not implemented.')
  }
  public requestSetTxnAuthorAgreementAcceptance(
    options: RequestSetTxnAuthorAgreementAcceptanceOptions & { requestHandle: number }
  ): void {
    throw new Error('Method not implemented.')
  }
}
