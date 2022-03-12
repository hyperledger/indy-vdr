type PoolHandle = number
type RequestHandle = number

export type Callback = (err: number) => void

export type CallbackWithResponse = (err: number, response: string) => void

export interface IndyVdrNativeBindings {
  version(): string

  get_current_error(): string

  set_config(options: { config: string }): void

  set_default_logger(): void

  set_protocol_version(options: { version: number }): void

  set_socks_proxy(options: { socks_proxy: string }): void

  build_acceptance_mechanisms_request(options: {
    submitter_did: string
    aml: string
    version: string
    aml_context?: string
  }): RequestHandle

  build_get_acceptance_mechanisms_request(options: {
    submitter_did?: string
    timestamp?: number
    version?: string
  }): RequestHandle

  build_attrib_request(options: {
    submitter_did: string
    target_did: string
    hash?: string
    raw?: string
    enc?: string
  }): RequestHandle

  build_get_attrib_request(options: {
    submitter_did?: string
    target_did: string
    raw?: string
    hash?: string
    enc?: string
  }): RequestHandle

  build_cred_def_request(options: { submitter_did: string; cred_def: string }): RequestHandle

  build_get_cred_def_request(options: { submitter_did?: string; cred_def_id: string }): RequestHandle

  build_get_revoc_reg_def_request(options: { submitter_did?: string; revoc_reg_id: string }): RequestHandle

  build_get_revoc_reg_request(options: {
    submitter_did?: string
    revoc_reg_id: string
    timestamp: number
  }): RequestHandle

  build_get_revoc_reg_delta_request(options: {
    submitter_did?: string
    revoc_reg_id: string
    from_ts?: number
    to_ts: number
  }): RequestHandle

  build_revoc_reg_def_request(options: { submitter_did: string; revoc_reg_def: string }): RequestHandle

  build_custom_request(options: { request_json: string }): RequestHandle

  build_disable_all_txn_author_agreements_request(options: { submitter_did: string }): RequestHandle

  build_get_nym_request(options: { submitter_did?: string; dest: string }): RequestHandle

  build_get_schema_request(options: { submitter_did?: string; schema_id: string }): RequestHandle

  build_get_txn_author_agreement_request(options: { submitter_did?: string; data?: string }): RequestHandle

  build_get_txn_request(options: { submitter_did?: string; ledger_type: number; seq_no: number }): RequestHandle

  build_get_validator_info_request(options: { submitter_did: string }): RequestHandle

  build_nym_request(options: {
    submitter_did: string
    dest: string
    verkey?: string
    alias?: string
    role?: string
  }): RequestHandle

  build_revoc_reg_entry_request(options: {
    submitter_did: string
    revoc_reg_def_id: string
    revoc_reg_def_type: string
    revoc_reg_entry: string
  }): RequestHandle

  build_schema_request(options: { submitter_did: string; schema: string }): RequestHandle

  build_txn_author_agreement_request(options: {
    submitter_did: string
    text?: string
    version: string
    ratification_ts?: number
    retirement_ts?: number
  }): RequestHandle

  build_rich_schema_request(options: {
    submitter_did: string
    rs_id: string
    rs_content: string
    rs_name: string
    rs_version: string
    rs_type: string
    ver: string
  }): RequestHandle

  build_get_rich_schema_object_by_id_request(options: { submitter_did: string; rs_id: string }): RequestHandle

  build_get_rich_schema_object_by_metadata_request(options: {
    submitter_did: string
    rs_type: string
    rs_name: string
    rs_version: string
  }): RequestHandle

  pool_create(options: { params: string }): PoolHandle

  pool_refresh(options: { pool_handle: PoolHandle; cb: Callback }): void

  pool_get_status(options: { pool_handle: PoolHandle; cb: CallbackWithResponse }): void

  pool_get_transactions(options: { pool_handle: PoolHandle; cb: CallbackWithResponse }): void

  pool_get_verifiers(options: { pool_handle: PoolHandle; cb: CallbackWithResponse }): void

  pool_submit_action(options: {
    pool_handle: PoolHandle
    request_handle: number
    nodes?: string
    timeout?: number
    cb: CallbackWithResponse
  }): void

  pool_submit_request(options: { pool_handle: PoolHandle; request_handle: number; cb: CallbackWithResponse }): void

  pool_close(options: { pool_handle: PoolHandle }): void

  prepare_txn_author_agreement_acceptance(options: {
    text?: string
    version?: string
    taa_digest?: string
    acc_mech_type: string
    time: number
  }): string

  request_free(options: { request_handle: number }): void

  request_get_body(options: { request_handle: number }): string

  request_get_signature_input(options: { request_handle: number }): string

  request_set_endorser(options: { request_handle: number; endorser: string }): void

  request_set_multi_signature(options: {
    request_handle: number
    identifier: string
    signature: number
    signature_len: number
  }): void

  request_set_signature(options: { request_handle: number; signature: number; signature_len: number }): void

  request_set_txn_author_agreement_acceptance(options: { request_handle: number; acceptance: string }): void
}
