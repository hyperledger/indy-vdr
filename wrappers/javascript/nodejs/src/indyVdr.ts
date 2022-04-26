import type { NativeCallback, NativeCallbackWithResponse, SerializedOptions } from './utils'
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

import { readCString, deref, address } from 'ref-napi'

import { handleError } from './error'
import { nativeIndyVdr } from './lib'
import {
  serializeArguments,
  allocateHandleBuffer,
  allocateStringBuffer,
  toNativeCallback,
  toNativeCallbackWithResponse,
  uint8ArrayToByteBuffer,
} from './utils'
import { PerformanceEntry } from 'perf_hooks'

export class NodeJSIndyVdr implements IndyVdr {
  private promisify = async (method: (nativeCallbackPtr: Buffer, id: number) => void): Promise<void> => {
    return new Promise((resolve, reject) => {
      const cb: NativeCallback = (id, errorCode) => {
        clearTimeout(id as unknown as NodeJS.Timeout)
        if (errorCode) reject(this.getCurrentError())
        resolve()
      }
      const { nativeCallback, id } = toNativeCallback(cb)
      method(nativeCallback, +id)
    })
  }

  private promisifyWithResponse = async <T>(
    method: (nativeCallbackWithResponsePtr: Buffer, id: number) => void,
    isStream = false
  ): Promise<T> => {
    return new Promise((resolve, reject) => {
      const cb: NativeCallbackWithResponse = (id, errorCode, response) => {
        clearTimeout(id as unknown as NodeJS.Timeout)
        if (errorCode) reject(this.getCurrentError())

        try {
          //this is required to add array brackets, and commas, to an invalid json object that
          // should be a list
          const mappedResponse = isStream ? '[' + response.replace(/\n/g, ',') + ']' : response
          resolve(JSON.parse(mappedResponse) as T)
        } catch (error) {
          reject(error)
        }
      }
      const { nativeCallback, id } = toNativeCallbackWithResponse(cb)
      method(nativeCallback, +id)
    })
  }

  public getCurrentError(): string {
    const error = allocateStringBuffer()
    handleError(nativeIndyVdr.indy_vdr_get_current_error(error))
    return error.deref() as string
  }

  public version(): string {
    return nativeIndyVdr.indy_vdr_version()
  }

  public setConfig(options: { config: Record<string, unknown> }): void {
    const { config } = serializeArguments(options) as SerializedOptions<typeof options>
    handleError(nativeIndyVdr.indy_vdr_set_config(config))
  }

  public setDefaultLogger(): void {
    handleError(nativeIndyVdr.indy_vdr_set_default_logger())
  }

  public setProtocolVersion(options: { version: number }): void {
    const { version } = serializeArguments(options) as SerializedOptions<typeof options>
    handleError(nativeIndyVdr.indy_vdr_set_protocol_version(version))
  }

  public setSocksProxy(options: { socksProxy: string }): void {
    const { socksProxy } = serializeArguments(options) as SerializedOptions<typeof options>
    handleError(nativeIndyVdr.indy_vdr_set_socks_proxy(socksProxy))
  }

  public buildAcceptanceMechanismsRequest(options: AcceptanceMechanismsRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { version, aml, submitterDid, amlContext } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(
      nativeIndyVdr.indy_vdr_build_acceptance_mechanisms_request(submitterDid, aml, version, amlContext, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildGetAcceptanceMechanismsRequest(options: GetAcceptanceMechanismsRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { submitterDid, timestamp, version } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(
      nativeIndyVdr.indy_vdr_build_get_acceptance_mechanisms_request(submitterDid, timestamp, version, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildAttribRequest(options: AttribRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { submitterDid, targetDid, raw, hash, enc } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_attrib_request(submitterDid, targetDid, raw, hash, enc, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetAttribRequest(options: GetAttribRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { submitterDid, targetDid, raw, hash, enc } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_get_attrib_request(submitterDid, targetDid, raw, hash, enc, requestHandle))

    return requestHandle.deref() as number
  }

  public buildCredDefRequest(options: CredentialDefinitionRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { credentialDefinition, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_cred_def_request(submitterDid, credentialDefinition, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetCredDefRequest(options: GetCredentialDefinitionRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { credentialDefinitionId, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_get_cred_def_request(submitterDid, credentialDefinitionId, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetRevocRegDefRequest(options: GetRevocationRegistryDefinitionRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { revocationRegistryId, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(
      nativeIndyVdr.indy_vdr_build_get_revoc_reg_def_request(submitterDid, revocationRegistryId, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildGetRevocRegRequest(options: GetRevocationRegistryRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { revocationRegistryId, timestamp, submitterDid } = serializeArguments(options) as SerializedOptions<
      typeof options
    >

    handleError(
      nativeIndyVdr.indy_vdr_build_get_revoc_reg_request(submitterDid, revocationRegistryId, timestamp, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildGetRevocRegDeltaRequest(options: GetRevocationRegistryDeltaRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { revocationRegistryId, toTs, fromTs, submitterDid } = serializeArguments(options) as SerializedOptions<
      typeof options
    >

    handleError(
      nativeIndyVdr.indy_vdr_build_get_revoc_reg_delta_request(
        submitterDid,
        revocationRegistryId,
        fromTs,
        toTs,
        requestHandle
      )
    )

    return requestHandle.deref() as number
  }

  public buildRevocRegDefRequest(options: RevocationRegistryDefinitionRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { revocationRegistryId, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_revoc_reg_def_request(submitterDid, revocationRegistryId, requestHandle))

    return requestHandle.deref() as number
  }

  public buildCustomRequest(options: CustomRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { customRequest } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_custom_request(customRequest, requestHandle))

    return requestHandle.deref() as number
  }

  public buildDisableAllTxnAuthorAgreementsRequest(
    options: DisableAllTransactionAuthorAgreementsRequestOptions
  ): number {
    const requestHandle = allocateHandleBuffer()
    const { submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_disable_all_txn_author_agreements_request(submitterDid, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetNymRequest(options: GetNymRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { dest, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_get_nym_request(submitterDid, dest, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetSchemaRequest(options: GetSchemaRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { schemaId, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_get_schema_request(submitterDid, schemaId, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetTxnAuthorAgreementRequest(options: GetTransactionAuthorAgreementRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { data, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_get_txn_author_agreement_request(submitterDid, data, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetTxnRequest(options: GetTransactionRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { ledgerType, seqNo, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_get_txn_request(submitterDid, ledgerType, seqNo, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetValidatorInfoRequest(options: GetValidatorInfoRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_get_validator_info_request(submitterDid, requestHandle))

    return requestHandle.deref() as number
  }

  public buildNymRequest(options: NymRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { dest, submitterDid, alias, role, verkey } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_nym_request(submitterDid, dest, alias, role, verkey, requestHandle))

    return requestHandle.deref() as number
  }

  public buildRevocRegEntryRequest(options: RevocationRegistryEntryRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { revocationRegistryDefinitionId, revocationRegistryDefinitionType, revocationRegistryEntry, submitterDid } =
      serializeArguments(options) as SerializedOptions<typeof options>

    handleError(
      nativeIndyVdr.indy_vdr_build_revoc_reg_entry_request(
        submitterDid,
        revocationRegistryDefinitionId,
        revocationRegistryDefinitionType,
        revocationRegistryEntry,
        requestHandle
      )
    )

    return requestHandle.deref() as number
  }

  public buildSchemaRequest(options: SchemaRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { schema, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_build_schema_request(submitterDid, schema, requestHandle))

    return requestHandle.deref() as number
  }

  public buildTxnAuthorAgreementRequest(options: TransactionAuthorAgreementRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { submitterDid, version, ratificationTs, retirementTs, text } = serializeArguments(
      options
    ) as SerializedOptions<typeof options>

    handleError(
      nativeIndyVdr.indy_vdr_build_txn_author_agreement_request(
        submitterDid,
        text,
        version,
        ratificationTs,
        retirementTs,
        requestHandle
      )
    )

    return requestHandle.deref() as number
  }

  public buildRichSchemaRequest(options: RichSchemaRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { content, id, name, submitterDid, type, ver, version } = serializeArguments(options) as SerializedOptions<
      typeof options
    >

    // handleError(nativeIndyVdr.indy_vdr_build_rich_schema_request())

    return requestHandle.deref() as number
  }

  public buildGetRichSchemaObjectByIdRequest(options: GetRichSchemaObjectByIdRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { id, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>

    // handleError(nativeIndyVdr.indy_vdr_build_get_rich_schema_object_by_id_request())

    return requestHandle.deref() as number
  }

  public buildGetRichSchemaObjectByMetadataRequest(options: GetRichSchemaObjectByMetadataRequestOptions): number {
    const requestHandle = allocateHandleBuffer()
    const { name, submitterDid, type, version } = serializeArguments(options) as SerializedOptions<typeof options>

    // handleError(nativeIndyVdr.indy_vdr_build_get_rich_schema_object_by_metadata_request())

    return requestHandle.deref() as number
  }

  public poolCreate(options: PoolCreateOptions): number {
    const poolHandle = allocateHandleBuffer()
    const { parameters } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_pool_create(parameters, poolHandle))

    return poolHandle.deref() as number
  }

  public async poolRefresh(options: { poolHandle: number }): Promise<void> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    return this.promisify((cbPtr, id) => nativeIndyVdr.indy_vdr_pool_refresh(poolHandle, cbPtr, id))
  }

  public poolGetStatus(options: { poolHandle: number }): Promise<PoolStatus> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    return this.promisifyWithResponse((cbPtr, id) => nativeIndyVdr.indy_vdr_pool_get_status(poolHandle, cbPtr, id))
  }

  public async poolGetTransactions(options: { poolHandle: number }): Promise<Transactions> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    return this.promisifyWithResponse(
      (cbPtr, id) => nativeIndyVdr.indy_vdr_pool_get_transactions(poolHandle, cbPtr, id),
      true
    )
  }

  public async poolGetVerifiers(options: { poolHandle: number }): Promise<Verifiers> {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    return this.promisifyWithResponse(
      (cbPtr, id) => nativeIndyVdr.indy_vdr_pool_get_verifiers(poolHandle, cbPtr, id),
      true
    )
  }

  public async poolSubmitAction(options: PoolSubmitActionOptions & { poolHandle: number }): Promise<string> {
    const { requestHandle, poolHandle, nodes, timeout } = serializeArguments(options) as SerializedOptions<
      typeof options
    >

    return this.promisifyWithResponse((cbPtr, id) =>
      nativeIndyVdr.indy_vdr_pool_submit_action(poolHandle, requestHandle, nodes, timeout, cbPtr, id)
    )
  }

  public async poolSubmitRequest(options: PoolSubmitRequestOptions & { poolHandle: number }): Promise<string> {
    const { requestHandle, poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    return this.promisifyWithResponse((cbPtr, id) =>
      nativeIndyVdr.indy_vdr_pool_submit_request(poolHandle, requestHandle, cbPtr, id)
    )
  }

  public poolClose(options: { poolHandle: number }): void {
    const { poolHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_pool_close(poolHandle))
  }

  public prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string {
    const output = allocateStringBuffer()
    const { accMechType, time, taaDigest, text, version } = serializeArguments(options) as SerializedOptions<
      typeof options
    >

    handleError(
      nativeIndyVdr.indy_vdr_prepare_txn_author_agreement_acceptance(
        text,
        version,
        taaDigest,
        accMechType,
        time,
        output
      )
    )

    return output.deref() as string
  }

  public requestFree(options: { requestHandle: number }): void {
    const { requestHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_request_free(requestHandle))
  }

  public requestGetBody<T extends Record<string, unknown>>(options: { requestHandle: number }): T {
    const output = allocateStringBuffer()
    const { requestHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_request_get_body(requestHandle, output))

    return JSON.parse(output.deref() as string) as T
  }

  public requestGetSignatureInput(options: { requestHandle: number }): string {
    const output = allocateStringBuffer()
    const { requestHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_request_get_signature_input(requestHandle, output))

    return output.deref() as string
  }

  public requestSetEndorser(options: RequestSetEndorserOptions & { requestHandle: number }): void {
    const { endorser, requestHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_request_set_endorser(requestHandle, endorser))
  }

  public requestSetMultiSignature(options: RequestSetMultiSignatureOptions & { requestHandle: number }): void {
    const { identifier, requestHandle, signature } = serializeArguments(options) as SerializedOptions<typeof options>
    // TODO: move to serialize
    const convertedSignature = uint8ArrayToByteBuffer(Buffer.from(options.signature))

    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    handleError(nativeIndyVdr.indy_vdr_request_set_multi_signature(requestHandle, identifier, convertedSignature))
  }

  public requestSetSignature(options: RequestSetSignatureOptions & { requestHandle: number }): void {
    const { requestHandle, signature } = serializeArguments(options) as SerializedOptions<typeof options>
    // TODO: move to serialize
    const convertedSignature = uint8ArrayToByteBuffer(Buffer.from(options.signature))

    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    handleError(nativeIndyVdr.indy_vdr_request_set_signature(requestHandle, convertedSignature))
  }

  public requestSetTxnAuthorAgreementAcceptance(
    options: RequestSetTxnAuthorAgreementAcceptanceOptions & { requestHandle: number }
  ): void {
    const { acceptance, requestHandle } = serializeArguments(options) as SerializedOptions<typeof options>

    handleError(nativeIndyVdr.indy_vdr_request_set_txn_author_agreement_acceptance(requestHandle, acceptance))
  }
}
