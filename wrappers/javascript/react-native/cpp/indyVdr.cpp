#include <indyVdr.h>

using namespace turboModuleUtility;

namespace indyVdr {

RequestHandle requestHandle = 0;
PoolHandle poolHandle = 0;

RequestHandle getNewRequestHandle() {
  requestHandle++;

  return int(requestHandle);
}

PoolHandle getNewPoolHandle() {
  poolHandle++;

  return int(poolHandle);
}

jsi::Value version(jsi::Runtime &rt, jsi::Object options) {
  return jsi::String::createFromAscii(rt, indy_vdr_version());
}

jsi::Value getCurrentError(jsi::Runtime &rt) {
  const char *errorMessage;
  indy_vdr_get_current_error(&errorMessage);

  return jsi::String::createFromAscii(rt, errorMessage);
};

jsi::Value setConfig(jsi::Runtime &rt, jsi::Object options) {
  std::string config = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "config"));

  ErrorCode code = indy_vdr_set_config(config.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value setDefaultLogger(jsi::Runtime &rt, jsi::Object options) {
  ErrorCode code = indy_vdr_set_default_logger();
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value setProtocolVersion(jsi::Runtime &rt, jsi::Object options) {
  int64_t version = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "version"));

  ErrorCode code = indy_vdr_set_protocol_version(version);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value setSocksProxy(jsi::Runtime &rt, jsi::Object options) {
  std::string socksProxy = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "socksProxy"));

  ErrorCode code = indy_vdr_set_socks_proxy(socksProxy.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value buildAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                            jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string aml = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "aml"));
  std::string version = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "version"));
  std::string amlContext = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "amlContext"), true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_acceptance_mechanisms_request(
      submitterDid.c_str(), aml.c_str(), version.c_str(),
      amlContext.length() ? amlContext.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value((int)requestHandle);
};

jsi::Value buildGetAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                               jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  int64_t timestamp = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "timestamp"), true);
  std::string version = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "version"), true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_acceptance_mechanisms_request(
      submitterDid.c_str(), timestamp,
      version.length() > 0 ? version.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return (int)requestHandle;
};

jsi::Value buildAttribRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string targetDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "targetDid"));
  std::string hash = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "hash"), true);
  std::string raw = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "raw"), true);
  std::string enc = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "enc"), true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_attrib_request(
      submitterDid.c_str(), targetDid.c_str(),
      hash.length() > 0 ? hash.c_str() : nullptr,
      raw.length() > 0 ? raw.c_str() : nullptr,
      enc.length() > 0 ? enc.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetAttribRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string targetDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "targetDid"));
  std::string hash = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "hash"), true);
  std::string raw = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "raw"), true);
  std::string enc = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "enc"), true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_attrib_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      targetDid.c_str(), hash.length() > 0 ? hash.c_str() : nullptr,
      raw.length() > 0 ? raw.c_str() : nullptr,
      enc.length() > 0 ? enc.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildCredDefRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string credDef = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "credDef"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_cred_def_request(
      submitterDid.c_str(), credDef.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetCredDefRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string credDefId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "credDefId"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_cred_def_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      credDefId.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string revocRegId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "revocRegId"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_revoc_reg_def_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      revocRegId.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRevocRegRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string revocRegId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "revocRegId"));
  int64_t timestamp = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "timestamp"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_revoc_reg_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      revocRegId.c_str(), timestamp, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRevocRegDeltaRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string revocRegId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "revocRegId"));
  int64_t fromTs = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "fromTs"), true);
  int64_t toTs = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "toTs"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_revoc_reg_delta_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      revocRegId.c_str(), fromTs, toTs, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string revocRegId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "revocRegId"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_revoc_reg_def_request(
      submitterDid.c_str(), revocRegId.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildCustomRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string requestJson = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "customRequest"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code =
      indy_vdr_build_custom_request(requestJson.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildDisableAllTxnAuthorAgreementsRequest(jsi::Runtime &rt,
                                                     jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_disable_all_txn_author_agreements_request(
      submitterDid.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetNymRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string dest = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "dest"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_nym_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr, dest.c_str(),
      &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetSchemaRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string schemaId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "schemaId"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_schema_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      schemaId.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetTxnAuthorAgreementRequest(jsi::Runtime &rt,
                                             jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  std::string data = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "data"), true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_txn_author_agreement_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      data.length() > 0 ? data.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetTxnRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"), true);
  int32_t ledgerType = turboModuleUtility::jsiToValue<int32_t>(
      rt, options.getProperty(rt, "ledgerType"));
  int32_t seqNo = turboModuleUtility::jsiToValue<int32_t>(
      rt, options.getProperty(rt, "seqNo"));
  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_txn_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr, ledgerType,
      seqNo, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetValidatorInfoRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_validator_info_request(
      submitterDid.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildNymRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string dest = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "dest"));
  std::string verkey = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "verkey"), true);
  std::string alias = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "alias"), true);
  std::string role = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "role"), true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_nym_request(
      submitterDid.c_str(), dest.c_str(),
      verkey.length() > 0 ? verkey.c_str() : nullptr,
      alias.length() > 0 ? alias.c_str() : nullptr,
      role.length() > 0 ? role.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildRevocRegEntryRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string revocRegDefId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "revocRegDefId"));
  std::string revocRegDefType = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "revocRegDefType"));
  std::string revocRegEntry = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "revocRegEntry"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_revoc_reg_entry_request(
      submitterDid.c_str(), revocRegDefId.c_str(), revocRegDefType.c_str(),
      revocRegEntry.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildSchemaRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string schema = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "schema"));

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_schema_request(
      submitterDid.c_str(), schema.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildTxnAuthorAgreementRequest(jsi::Runtime &rt,
                                          jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string text = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "text"), true);
  std::string version = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "version"));
  int64_t ratificationTs = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "ratificationTs"), true);
  int64_t retirementTs = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "retirementTs"), true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_txn_author_agreement_request(
      submitterDid.c_str(), text.length() > 0 ? text.c_str() : nullptr,
      version.c_str(), ratificationTs, retirementTs, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildRichSchemaRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string rsId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsId"));
  std::string rsContent = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsContent"));
  std::string rsName = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsName"));
  std::string rsVersion = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsVersion"));
  std::string rsType = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsType"));
  std::string ver = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "ver"));

  RequestHandle requestHandle = getNewRequestHandle();
  // indy_vdr_build_rich_schema_request(
  //     submitterDid.c_str(), rsId.c_str(), rsContent.c_str(), rsName.c_str(),
  //     rsVersion.c_str(), rsType.c_str(), ver.c_str(), &requestHandle);
  ErrorCode code;
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRichSchemaObjectByIdRequest(jsi::Runtime &rt,
                                               jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string rsId = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsId"));

  RequestHandle requestHandle = getNewRequestHandle();
  // ErrorCode code =
  // indy_vdr_build_get_rich_schema_object_by_id_request(submitterDid.c_str(),
  // rsId.c_str(),
  //                                            &requestHandle);
  ErrorCode code;
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRichSchemaObjectByMetadataRequest(jsi::Runtime &rt,
                                                     jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "submitterDid"));
  std::string rsType = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsType"));
  std::string rsName = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsName"));
  std::string rsVersion = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "rsVersion"));

  RequestHandle requestHandle = getNewRequestHandle();
  // ErrorCode code = indy_vdr_build_get_rich_schema_object_by_metadata_request(
  //     submitterDid.c_str(), rsType.c_str(), rsName.c_str(),
  //     rsVersion.c_str(), &requestHandle);
  ErrorCode code;
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value poolCreate(jsi::Runtime &rt, jsi::Object options) {
  std::string params = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "params"));

  PoolHandle poolHandle = getNewPoolHandle();
  ErrorCode code = indy_vdr_pool_create(params.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(poolHandle);
};

jsi::Value poolRefresh(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "poolHandle"));

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  indy_vdr_pool_refresh(poolHandle, turboModuleUtility::callback,
                        CallbackId(state));

  return jsi::Value::null();
};

jsi::Value poolGetStatus(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "poolHandle"));

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  indy_vdr_pool_get_status(poolHandle, turboModuleUtility::callbackWithResponse,
                           CallbackId(state));

  return jsi::Value::null();
};

jsi::Value poolGetTransactions(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "poolHandle"));

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  indy_vdr_pool_get_transactions(
      poolHandle, turboModuleUtility::callbackWithResponse, CallbackId(state));

  return jsi::Value::null();
};

jsi::Value poolGetVerifiers(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "poolHandle"));

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  indy_vdr_pool_get_verifiers(
      poolHandle, turboModuleUtility::callbackWithResponse, CallbackId(state));

  return jsi::Value::null();
};

jsi::Value poolSubmitAction(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "poolHandle"));
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));
  std::string nodes = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "nodes"), true);
  int32_t timeout = turboModuleUtility::jsiToValue<int32_t>(
      rt, options.getProperty(rt, "timeout"), true);

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  indy_vdr_pool_submit_action(
      poolHandle, requestHandle, nodes.length() > 0 ? nodes.c_str() : nullptr,
      timeout, turboModuleUtility::callbackWithResponse, CallbackId(state));

  return jsi::Value::null();
};

jsi::Value poolSubmitRequest(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "poolHandle"));
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  indy_vdr_pool_submit_request(poolHandle, requestHandle,
                               turboModuleUtility::callbackWithResponse,
                               CallbackId(state));

  return jsi::Value::null();
};

jsi::Value poolClose(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "poolHandle"));

  ErrorCode code = indy_vdr_pool_close(poolHandle);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value prepareTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
                                               jsi::Object options) {
  std::string text = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "text"));
  std::string version = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "version"));
  std::string taaDigest = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "taaDigest"));
  std::string accMechType = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "accMechType"));
  int32_t time = turboModuleUtility::jsiToValue<int32_t>(
      rt, options.getProperty(rt, "time"));

  const char *output;
  ErrorCode code = indy_vdr_prepare_txn_author_agreement_acceptance(
      text.c_str(), version.c_str(), taaDigest.c_str(), accMechType.c_str(),
      time, &output);
  turboModuleUtility::handleError(rt, code);

  return jsi::String::createFromAscii(rt, output);
};

jsi::Value requestFree(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));

  ErrorCode code = indy_vdr_request_free(requestHandle);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value requestGetBody(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));

  const char *bodyP;
  ErrorCode code = indy_vdr_request_get_body(requestHandle, &bodyP);
  turboModuleUtility::handleError(rt, code);

  return jsi::String::createFromAscii(rt, bodyP);
};

jsi::Value requestGetSignatureInput(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));

  const char *inputP;
  ErrorCode code = indy_vdr_request_get_signature_input(requestHandle, &inputP);
  turboModuleUtility::handleError(rt, code);

  return jsi::String::createFromAscii(rt, inputP);
};

jsi::Value requestSetEndorser(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));
  std::string endorser = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "endorser"));

  ErrorCode code =
      indy_vdr_request_set_endorser(requestHandle, endorser.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};
jsi::Value requestSetMultiSignature(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));
  std::string identifier = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "identifier"));
  ByteBuffer signature = turboModuleUtility::jsiToValue<ByteBuffer>(
      rt, options.getProperty(rt, "signature"));

  ErrorCode code = indy_vdr_request_set_multi_signature(
      requestHandle, identifier.c_str(), signature);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value requestSetSignature(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));
  ByteBuffer signature = turboModuleUtility::jsiToValue<ByteBuffer>(
      rt, options.getProperty(rt, "signature"));

  ErrorCode code = indy_vdr_request_set_signature(requestHandle, signature);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value requestSetTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
                                                  jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
          rt, options.getProperty(rt, "requestHandle"));
  std::string acceptance = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "acceptance"));

  ErrorCode code = indy_vdr_request_set_txn_author_agreement_acceptance(
      requestHandle, acceptance.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

} // namespace indyVdr
