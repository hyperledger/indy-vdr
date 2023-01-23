import type { NativeCallback, NativeCallbackWithResponse } from './ffi'
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
  SchemaRequestOptions,
  TransactionAuthorAgreementRequestOptions,
  Transactions,
  Verifiers,
} from '@hyperledger/indy-vdr-shared'

import { handleError } from './error'
import {
  deallocateCallback,
  allocateHandle,
  allocateString,
  toNativeCallback,
  toNativeCallbackWithResponse,
  serializeArguments,
} from './ffi'
import { nativeIndyVdr } from './library'

export class NodeJSIndyVdr implements IndyVdr {
  private promisify = async (method: (nativeCallbackPtr: Buffer, id: number) => void): Promise<void> => {
    return new Promise((resolve, reject) => {
      const cb: NativeCallback = (id, errorCode) => {
        deallocateCallback(id)

        try {
          handleError(errorCode)
        } catch (e) {
          reject(e)
        }

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
        deallocateCallback(id)

        try {
          handleError(errorCode)
        } catch (e) {
          return reject(e)
        }

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
    const error = allocateString()
    handleError(nativeIndyVdr.indy_vdr_get_current_error(error))
    return error.deref() as string
  }

  public version(): string {
    return nativeIndyVdr.indy_vdr_version()
  }

  public setConfig(options: { config: Record<string, unknown> }): void {
    const { config } = serializeArguments(options)
    handleError(nativeIndyVdr.indy_vdr_set_config(config))
  }

  public setDefaultLogger(): void {
    handleError(nativeIndyVdr.indy_vdr_set_default_logger())
  }

  public setProtocolVersion(options: { version: number }): void {
    const { version } = serializeArguments(options)
    handleError(nativeIndyVdr.indy_vdr_set_protocol_version(version))
  }

  public setSocksProxy(options: { socksProxy: string }): void {
    const { socksProxy } = serializeArguments(options)
    handleError(nativeIndyVdr.indy_vdr_set_socks_proxy(socksProxy))
  }

  public buildAcceptanceMechanismsRequest(options: AcceptanceMechanismsRequestOptions): number {
    const requestHandle = allocateHandle()
    const { version, aml, submitterDid, amlContext } = serializeArguments(options)

    handleError(
      nativeIndyVdr.indy_vdr_build_acceptance_mechanisms_request(submitterDid, aml, version, amlContext, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildGetAcceptanceMechanismsRequest(options: GetAcceptanceMechanismsRequestOptions): number {
    const requestHandle = allocateHandle()
    const { submitterDid, timestamp, version } = serializeArguments(options)

    // We cannot handle this step in the serialization. Indy-vdr expects a -1 for an undefined timestamp
    const convertedTimestamp = timestamp ?? -1

    handleError(
      nativeIndyVdr.indy_vdr_build_get_acceptance_mechanisms_request(
        submitterDid,
        convertedTimestamp,
        version,
        requestHandle
      )
    )

    return requestHandle.deref() as number
  }

  public buildAttribRequest(options: AttribRequestOptions): number {
    const requestHandle = allocateHandle()
    const { submitterDid, targetDid, raw, hash, enc } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_attrib_request(submitterDid, targetDid, hash, raw, enc, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetAttribRequest(options: GetAttribRequestOptions): number {
    const requestHandle = allocateHandle()
    const { submitterDid, targetDid, raw, hash, enc } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_get_attrib_request(submitterDid, targetDid, raw, hash, enc, requestHandle))

    return requestHandle.deref() as number
  }

  public buildCredDefRequest(options: CredentialDefinitionRequestOptions): number {
    const requestHandle = allocateHandle()
    const { credentialDefinition, submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_cred_def_request(submitterDid, credentialDefinition, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetCredDefRequest(options: GetCredentialDefinitionRequestOptions): number {
    const requestHandle = allocateHandle()
    const { credentialDefinitionId, submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_get_cred_def_request(submitterDid, credentialDefinitionId, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetRevocRegDefRequest(options: GetRevocationRegistryDefinitionRequestOptions): number {
    const requestHandle = allocateHandle()
    const { revocationRegistryId, submitterDid } = serializeArguments(options)

    handleError(
      nativeIndyVdr.indy_vdr_build_get_revoc_reg_def_request(submitterDid, revocationRegistryId, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildGetRevocRegRequest(options: GetRevocationRegistryRequestOptions): number {
    const requestHandle = allocateHandle()
    const { revocationRegistryId, timestamp, submitterDid } = serializeArguments(options)

    handleError(
      nativeIndyVdr.indy_vdr_build_get_revoc_reg_request(submitterDid, revocationRegistryId, timestamp, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildGetRevocRegDeltaRequest(options: GetRevocationRegistryDeltaRequestOptions): number {
    const requestHandle = allocateHandle()
    const { revocationRegistryId, toTs, fromTs, submitterDid } = serializeArguments(options)

    const convertedFromTs = fromTs ?? -1

    handleError(
      nativeIndyVdr.indy_vdr_build_get_revoc_reg_delta_request(
        submitterDid,
        revocationRegistryId,
        convertedFromTs,
        toTs,
        requestHandle
      )
    )

    return requestHandle.deref() as number
  }

  public buildRevocRegDefRequest(options: RevocationRegistryDefinitionRequestOptions): number {
    const requestHandle = allocateHandle()
    const { submitterDid, revocationRegistryDefinitionV1: revocationRegistryDefinition } = serializeArguments(options)

    handleError(
      nativeIndyVdr.indy_vdr_build_revoc_reg_def_request(submitterDid, revocationRegistryDefinition, requestHandle)
    )

    return requestHandle.deref() as number
  }

  public buildCustomRequest(options: CustomRequestOptions): number {
    const requestHandle = allocateHandle()
    const { customRequest } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_custom_request(customRequest, requestHandle))

    return requestHandle.deref() as number
  }

  public buildDisableAllTxnAuthorAgreementsRequest(
    options: DisableAllTransactionAuthorAgreementsRequestOptions
  ): number {
    const requestHandle = allocateHandle()
    const { submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_disable_all_txn_author_agreements_request(submitterDid, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetNymRequest(options: GetNymRequestOptions): number {
    const requestHandle = allocateHandle()
    const { dest, submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_get_nym_request(submitterDid, dest, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetSchemaRequest(options: GetSchemaRequestOptions): number {
    const requestHandle = allocateHandle()
    const { schemaId, submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_get_schema_request(submitterDid, schemaId, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetTxnAuthorAgreementRequest(options: GetTransactionAuthorAgreementRequestOptions): number {
    const requestHandle = allocateHandle()
    const { data, submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_get_txn_author_agreement_request(submitterDid, data, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetTxnRequest(options: GetTransactionRequestOptions): number {
    const requestHandle = allocateHandle()
    const { ledgerType, seqNo, submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_get_txn_request(submitterDid, ledgerType, seqNo, requestHandle))

    return requestHandle.deref() as number
  }

  public buildGetValidatorInfoRequest(options: GetValidatorInfoActionOptions): number {
    const requestHandle = allocateHandle()
    const { submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_get_validator_info_request(submitterDid, requestHandle))

    return requestHandle.deref() as number
  }

  public buildNymRequest(options: NymRequestOptions): number {
    const requestHandle = allocateHandle()
    const { dest, submitterDid, alias, role, verkey } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_nym_request(submitterDid, dest, verkey, alias, role, requestHandle))

    return requestHandle.deref() as number
  }

  public buildRevocRegEntryRequest(options: RevocationRegistryEntryRequestOptions): number {
    const requestHandle = allocateHandle()
    const { revocationRegistryDefinitionId, revocationRegistryDefinitionType, revocationRegistryEntry, submitterDid } =
      serializeArguments(options)

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
    const requestHandle = allocateHandle()
    const { schema, submitterDid } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_build_schema_request(submitterDid, schema, requestHandle))

    return requestHandle.deref() as number
  }

  public buildTxnAuthorAgreementRequest(options: TransactionAuthorAgreementRequestOptions): number {
    const requestHandle = allocateHandle()
    const { submitterDid, version, ratificationTs, retirementTs, text } = serializeArguments(options)

    const convertedRatificationTs = ratificationTs ?? -1
    const convertedRetirementTs = retirementTs ?? -1

    handleError(
      nativeIndyVdr.indy_vdr_build_txn_author_agreement_request(
        submitterDid,
        text,
        version,
        convertedRatificationTs,
        convertedRetirementTs,
        requestHandle
      )
    )

    return requestHandle.deref() as number
  }

  public poolCreate(options: PoolCreateOptions): number {
    const poolHandle = allocateHandle()
    const { parameters } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_pool_create(parameters, poolHandle))

    return poolHandle.deref() as number
  }

  public async poolRefresh(options: { poolHandle: number }): Promise<void> {
    const { poolHandle } = serializeArguments(options)

    return this.promisify((cbPtr, id) => nativeIndyVdr.indy_vdr_pool_refresh(poolHandle, cbPtr, id))
  }

  public poolGetStatus(options: { poolHandle: number }): Promise<PoolStatus> {
    const { poolHandle } = serializeArguments(options)

    return this.promisifyWithResponse((cbPtr, id) => nativeIndyVdr.indy_vdr_pool_get_status(poolHandle, cbPtr, id))
  }

  public async poolGetTransactions(options: { poolHandle: number }): Promise<Transactions> {
    const { poolHandle } = serializeArguments(options)

    return this.promisifyWithResponse<Transactions>(
      (cbPtr, id) => nativeIndyVdr.indy_vdr_pool_get_transactions(poolHandle, cbPtr, id),
      true
    )
  }

  public async poolGetVerifiers(options: { poolHandle: number }): Promise<Verifiers> {
    const { poolHandle } = serializeArguments(options)

    return this.promisifyWithResponse((cbPtr, id) => nativeIndyVdr.indy_vdr_pool_get_verifiers(poolHandle, cbPtr, id))
  }

  public async poolSubmitAction<T>(options: PoolSubmitActionOptions & { poolHandle: number }): Promise<T> {
    const { requestHandle, poolHandle, nodes, timeout } = serializeArguments(options)

    return this.promisifyWithResponse((cbPtr, id) =>
      nativeIndyVdr.indy_vdr_pool_submit_action(poolHandle, requestHandle, nodes, timeout, cbPtr, id)
    )
  }

  public async poolSubmitRequest<T>(options: PoolSubmitRequestOptions & { poolHandle: number }): Promise<T> {
    const { requestHandle, poolHandle } = serializeArguments(options)

    return this.promisifyWithResponse((cbPtr, id) =>
      nativeIndyVdr.indy_vdr_pool_submit_request(poolHandle, requestHandle, cbPtr, id)
    )
  }

  public poolClose(options: { poolHandle: number }): void {
    const { poolHandle } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_pool_close(poolHandle))
  }

  public prepareTxnAuthorAgreementAcceptance(options: PrepareTxnAuthorAgreementAcceptanceOptions): string {
    const output = allocateString()
    const { acceptanceMechanismType, time, taaDigest, text, version } = serializeArguments(options)

    handleError(
      nativeIndyVdr.indy_vdr_prepare_txn_author_agreement_acceptance(
        text,
        version,
        taaDigest,
        acceptanceMechanismType,
        time,
        output
      )
    )

    return output.deref() as string
  }

  public requestFree(options: { requestHandle: number }): void {
    const { requestHandle } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_request_free(requestHandle))
  }

  public requestGetBody<T extends Record<string, unknown>>(options: { requestHandle: number }): T {
    const output = allocateString()
    const { requestHandle } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_request_get_body(requestHandle, output))

    return JSON.parse(output.deref() as string) as T
  }

  public requestGetSignatureInput(options: { requestHandle: number }): string {
    const output = allocateString()
    const { requestHandle } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_request_get_signature_input(requestHandle, output))

    return output.deref() as string
  }

  public requestSetEndorser(options: RequestSetEndorserOptions & { requestHandle: number }): void {
    const { endorser, requestHandle } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_request_set_endorser(requestHandle, endorser))
  }

  public requestSetMultiSignature(options: RequestSetMultiSignatureOptions & { requestHandle: number }): void {
    const { identifier, requestHandle, signature } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_request_set_multi_signature(requestHandle, identifier, signature))
  }

  public requestSetSignature(options: RequestSetSignatureOptions & { requestHandle: number }): void {
    const { requestHandle, signature } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_request_set_signature(requestHandle, signature))
  }

  public requestSetTxnAuthorAgreementAcceptance(
    options: RequestSetTxnAuthorAgreementAcceptanceOptions & { requestHandle: number }
  ): void {
    const { acceptance, requestHandle } = serializeArguments(options)

    handleError(nativeIndyVdr.indy_vdr_request_set_txn_author_agreement_acceptance(requestHandle, acceptance))
  }
}
