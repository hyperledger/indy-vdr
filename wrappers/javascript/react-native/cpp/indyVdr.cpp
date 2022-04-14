#include <indyVdr.h>

namespace indyVdr {

RequestHandle requestHandle = 0;
RequestHandle poolHandle = 0;

RequestHandle getNewRequestHandle() {
  requestHandle++;
  return requestHandle;
}

PoolHandle getNewPoolHandle() {
  poolHandle++;
  return poolHandle;
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
      rt, options.getProperty(rt, "config"), "config");

  ErrorCode code = indy_vdr_set_config(config.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value setDefaultLogger(jsi::Runtime &rt) {
  ErrorCode code = indy_vdr_set_default_logger();
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value setProtocolVersion(jsi::Runtime &rt, jsi::Object options) {
  int64_t version = turboModuleUtility::jsiToValue<int64_t>(
      rt, options.getProperty(rt, "version"), "version");

  ErrorCode code = indy_vdr_set_protocol_version(version);
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value setSocksProxy(jsi::Runtime &rt, jsi::Object options) {
  std::string socksProxy = turboModuleUtility::jsiToValue<std::string>(
      rt, options.getProperty(rt, "socksProxy"), "socksProxy");

  ErrorCode code = indy_vdr_set_socks_proxy(socksProxy.c_str());
  turboModuleUtility::handleError(rt, code);

  return jsi::Value::null();
};

jsi::Value buildAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                       jsi::Object options){
    std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(rt, options.getProperty(rt, "submitterDid"), "submitterDid");
    std::string aml = turboModuleUtility::jsiToValue<std::string>(rt, options.getProperty(rt, "aml"), "aml");
    std::string version = turboModuleUtility::jsiToValue<std::string>(rt, options.getProperty(rt, "version"), "version");
    std::string amlContext = turboModuleUtility::jsiToValue<std::string>(rt, options.getProperty(rt, "amlContext"), "amlContext", true);
    RequestHandle requestHandle = getNewRequestHandle();

    ErrorCode code = indy_vdr_build_acceptance_mechanisms_request(submitterDid.c_str(),
                                                                aml.c_str(),
                                                                version.c_str(),
                                                                amlContext.length() ? amlContext.c_str() : nullptr,
                                                                &requestHandle);

    turboModuleUtility::handleError(rt, code);
    return jsi::Value((int)requestHandle);
};

jsi::Value buildGetAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                          jsi::Object options){
    std::string submitterDid = turboModuleUtility::jsiToValue<std::string>(rt, options.getProperty(rt, "submitterDid"), "submitterDid");
    int64_t timestamp = turboModuleUtility::jsiToValue<int64_t>(rt, options.getProperty(rt, "timestamp"), "timestamp", true);
    std::string version = turboModuleUtility::jsiToValue<std::string>(rt, options.getProperty(rt, "version"), "version", true);
    RequestHandle requestHandle = getNewRequestHandle();
    
    ErrorCode code = indy_vdr_build_get_acceptance_mechanisms_request(submitterDid.c_str(),
                                                                      timestamp,
                                                                      version.length() > 0 ? version.c_str() : nullptr,
                                                                      &requestHandle);

    turboModuleUtility::handleError(rt, code);
    return jsi::Value((int)requestHandle);
};

// jsi::Value buildAttribRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetAttribRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildCredDefRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetCredDefRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetRevocRegRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetRevocRegDeltaRequest(jsi::Runtime &rt,
//                                    jsi::Object options){
// 
// };
// 
// jsi::Value buildRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildCustomRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildDisableAllTxnAuthorAgreementsRequest(jsi::Runtime &rt,
//                                                 jsi::Object options){
// 
// };
// 
// jsi::Value buildGetNymRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetSchemaRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetTxnAuthorAgreementRequest(jsi::Runtime &rt,
//                                         jsi::Object options){
// 
// };
// 
// jsi::Value buildGetTxnRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetValidatorInfoRequest(jsi::Runtime &rt,
//                                    jsi::Object options){
// 
// };
// 
// jsi::Value buildNymRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildRevocRegEntryRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildSchemaRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildTxnAuthorAgreementRequest(jsi::Runtime &rt,
//                                      jsi::Object options){
// 
// };
// 
// jsi::Value buildRichSchemaRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value buildGetRichSchemaObjectByIdRequest(jsi::Runtime &rt,
//                                           jsi::Object options){
// 
// };
// 
// jsi::Value buildGetRichSchemaObjectByMetadataRequest(jsi::Runtime &rt,
//                                                 jsi::Object options){
// 
// };
// 
// jsi::Value poolCreate(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value poolRefresh(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value poolGetStatus(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value poolGetTransactions(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value poolGetVerifiers(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value poolSubmitAction(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value poolSubmitRequest(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value poolClose(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value requestSetEndorser(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value requestSetMultiSignature(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value requestSetSignature(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value requestSetTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
//                                                   jsi::Object options){
// 
// };
// 
// jsi::Value requestFree(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value prepareTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
//                                                 jsi::Object options){
// 
// };
// 
// jsi::Value requestGetBody(jsi::Runtime &rt, jsi::Object options){
// 
// };
// 
// jsi::Value requestGetSignatureInput(jsi::Runtime &rt, jsi::Object options){
// 
// };

} // namespace indyVdr
