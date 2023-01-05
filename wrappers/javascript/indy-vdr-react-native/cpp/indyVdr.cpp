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
  std::string config =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "config");

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
  int64_t version =
      turboModuleUtility::jsiToValue<int64_t>(rt, options, "version");

  ErrorCode code = indy_vdr_set_protocol_version(version);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value setSocksProxy(jsi::Runtime &rt, jsi::Object options) {
  std::string socksProxy =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "socksProxy");

  ErrorCode code = indy_vdr_set_socks_proxy(socksProxy.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value buildAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                            jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string aml =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "aml");
  std::string version =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "version");
  std::string amlContext = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "amlContext", true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_acceptance_mechanisms_request(
      submitterDid.c_str(), aml.c_str(), version.c_str(),
      amlContext.length() ? amlContext.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value((int)requestHandle);
};

jsi::Value buildGetAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                               jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  int64_t timestamp =
      turboModuleUtility::jsiToValue<int64_t>(rt, options, "timestamp", true);
  std::string version =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "version", true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_acceptance_mechanisms_request(
      submitterDid.c_str(), timestamp,
      version.length() > 0 ? version.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return (int)requestHandle;
};

jsi::Value buildAttribRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string targetDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "targetDid");
  std::string hash =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "hash", true);
  std::string raw =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "raw", true);
  std::string enc =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "enc", true);

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
      rt, options, "submitterDid", true);
  std::string targetDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "targetDid");
  std::string hash =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "hash", true);
  std::string raw =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "raw", true);
  std::string enc =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "enc", true);

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
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string credDef =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "credDef");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_cred_def_request(
      submitterDid.c_str(), credDef.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetCredDefRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "submitterDid", true);
  std::string credDefId =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "credDefId");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_cred_def_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      credDefId.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "submitterDid", true);
  std::string revocRegId =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "revocRegId");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_revoc_reg_def_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      revocRegId.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRevocRegRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "submitterDid", true);
  std::string revocRegId =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "revocRegId");
  int64_t timestamp =
      turboModuleUtility::jsiToValue<int64_t>(rt, options, "timestamp");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_revoc_reg_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      revocRegId.c_str(), timestamp, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetRevocRegDeltaRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "submitterDid", true);
  std::string revocRegId =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "revocRegId");
  int64_t fromTs =
      turboModuleUtility::jsiToValue<int64_t>(rt, options, "fromTs", true);
  int64_t toTs = turboModuleUtility::jsiToValue<int64_t>(rt, options, "toTs");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_revoc_reg_delta_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      revocRegId.c_str(), fromTs, toTs, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string revocRegId =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "revocRegId");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_revoc_reg_def_request(
      submitterDid.c_str(), revocRegId.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildCustomRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string requestJson =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "customRequest");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code =
      indy_vdr_build_custom_request(requestJson.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildDisableAllTxnAuthorAgreementsRequest(jsi::Runtime &rt,
                                                     jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_disable_all_txn_author_agreements_request(
      submitterDid.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetNymRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "submitterDid", true);
  std::string dest =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "dest");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_nym_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr, dest.c_str(),
      &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetSchemaRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "submitterDid", true);
  std::string schemaId =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "schemaId");

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
      rt, options, "submitterDid", true);
  std::string data =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "data", true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_txn_author_agreement_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr,
      data.length() > 0 ? data.c_str() : nullptr, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetTxnRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "submitterDid", true);
  int32_t ledgerType =
      turboModuleUtility::jsiToValue<int32_t>(rt, options, "ledgerType");
  int32_t seqNo = turboModuleUtility::jsiToValue<int32_t>(rt, options, "seqNo");
  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_txn_request(
      submitterDid.length() > 0 ? submitterDid.c_str() : nullptr, ledgerType,
      seqNo, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildGetValidatorInfoRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_get_validator_info_request(
      submitterDid.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildNymRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string dest =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "dest");
  std::string verkey =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "verkey", true);
  std::string alias =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "alias", true);
  std::string role =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "role", true);

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
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string revocRegDefId =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "revocRegDefId");
  std::string revocRegDefType = turboModuleUtility::jsiToValue<std::string>(
      rt, options, "revocRegDefType");
  std::string revocRegEntry =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "revocRegEntry");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_revoc_reg_entry_request(
      submitterDid.c_str(), revocRegDefId.c_str(), revocRegDefType.c_str(),
      revocRegEntry.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildSchemaRequest(jsi::Runtime &rt, jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string schema =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "schema");

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_schema_request(
      submitterDid.c_str(), schema.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value buildTxnAuthorAgreementRequest(jsi::Runtime &rt,
                                          jsi::Object options) {
  std::string submitterDid =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "submitterDid");
  std::string text =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "text", true);
  std::string version =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "version");
  int64_t ratificationTs = turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "ratificationTs", true);
  int64_t retirementTs = turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "retirementTs", true);

  RequestHandle requestHandle = getNewRequestHandle();
  ErrorCode code = indy_vdr_build_txn_author_agreement_request(
      submitterDid.c_str(), text.length() > 0 ? text.c_str() : nullptr,
      version.c_str(), ratificationTs, retirementTs, &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(requestHandle);
};

jsi::Value poolCreate(jsi::Runtime &rt, jsi::Object options) {
  std::string params =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "parameters");

  PoolHandle poolHandle = getNewPoolHandle();
  ErrorCode code = indy_vdr_pool_create(params.c_str(), &requestHandle);
  turboModuleUtility::handleError(rt, code);

  return int(poolHandle);
};

jsi::Value poolRefresh(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "poolHandle");

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  ErrorCode code = indy_vdr_pool_refresh(
      poolHandle, turboModuleUtility::callback, CallbackId(state));
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value poolGetStatus(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "poolHandle");

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  ErrorCode code = indy_vdr_pool_get_status(
      poolHandle, turboModuleUtility::callbackWithResponse, CallbackId(state));
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value poolGetTransactions(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "poolHandle");

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  ErrorCode code = indy_vdr_pool_get_transactions(
      poolHandle, turboModuleUtility::callbackWithResponse, CallbackId(state));
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value poolGetVerifiers(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "poolHandle");

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  ErrorCode code = indy_vdr_pool_get_verifiers(
      poolHandle, turboModuleUtility::callbackWithResponse, CallbackId(state));
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value poolSubmitAction(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "poolHandle");
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");
  std::string nodes =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "nodes", true);
  int32_t timeout =
      turboModuleUtility::jsiToValue<int32_t>(rt, options, "timeout", true);

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  ErrorCode code = indy_vdr_pool_submit_action(
      poolHandle, requestHandle, nodes.length() > 0 ? nodes.c_str() : nullptr,
      timeout, turboModuleUtility::callbackWithResponse, CallbackId(state));
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value poolSubmitRequest(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "poolHandle");
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");

  jsi::Function cb = options.getPropertyAsFunction(rt, "cb");
  State *state = new State(&cb);
  state->rt = &rt;
  ErrorCode code = indy_vdr_pool_submit_request(
      poolHandle, requestHandle, turboModuleUtility::callbackWithResponse,
      CallbackId(state));
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value poolClose(jsi::Runtime &rt, jsi::Object options) {
  PoolHandle poolHandle = (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(
      rt, options, "poolHandle");

  ErrorCode code = indy_vdr_pool_close(poolHandle);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value prepareTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
                                               jsi::Object options) {
  std::string text =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "text");
  std::string version =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "version");
  std::string taaDigest =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "taaDigest");
  std::string acceptanceMechanismType =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "acceptanceMechanismType");
  int32_t time = turboModuleUtility::jsiToValue<int32_t>(rt, options, "time");

  const char *output;
  ErrorCode code = indy_vdr_prepare_txn_author_agreement_acceptance(
      text.c_str(), version.c_str(), taaDigest.c_str(), acceptanceMechanismType.c_str(),
      time, &output);
  turboModuleUtility::handleError(rt, code);

  return jsi::String::createFromAscii(rt, output);
};

jsi::Value requestFree(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");

  ErrorCode code = indy_vdr_request_free(requestHandle);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value requestGetBody(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");

  const char *bodyP;
  ErrorCode code = indy_vdr_request_get_body(requestHandle, &bodyP);
  turboModuleUtility::handleError(rt, code);

  return jsi::String::createFromAscii(rt, bodyP);
};

jsi::Value requestGetSignatureInput(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");

  const char *inputP;
  ErrorCode code = indy_vdr_request_get_signature_input(requestHandle, &inputP);
  turboModuleUtility::handleError(rt, code);

  return jsi::String::createFromAscii(rt, inputP);
};

jsi::Value requestSetEndorser(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");
  std::string endorser =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "endorser");

  ErrorCode code =
      indy_vdr_request_set_endorser(requestHandle, endorser.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};
jsi::Value requestSetMultiSignature(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");
  std::string identifier =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "identifier");
  ByteBuffer signature =
      turboModuleUtility::jsiToValue<ByteBuffer>(rt, options, "signature");

  ErrorCode code = indy_vdr_request_set_multi_signature(
      requestHandle, identifier.c_str(), signature);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value requestSetSignature(jsi::Runtime &rt, jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");
  ByteBuffer signature =
      turboModuleUtility::jsiToValue<ByteBuffer>(rt, options, "signature");

  ErrorCode code = indy_vdr_request_set_signature(requestHandle, signature);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value requestSetTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
                                                  jsi::Object options) {
  RequestHandle requestHandle =
      (uintptr_t)turboModuleUtility::jsiToValue<int64_t>(rt, options,
                                                         "requestHandle");
  std::string acceptance =
      turboModuleUtility::jsiToValue<std::string>(rt, options, "acceptance");

  ErrorCode code = indy_vdr_request_set_txn_author_agreement_acceptance(
      requestHandle, acceptance.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

} // namespace indyVdr
