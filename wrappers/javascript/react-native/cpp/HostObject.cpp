#include <HostObject.h>
#include <algorithm>
#include <vector>

TurboModuleHostObject::TurboModuleHostObject(jsi::Runtime &rt) { return; }

FunctionMap TurboModuleHostObject::functionMapping(jsi::Runtime &rt) {
  FunctionMap fMap;
  fMap.insert(std::make_pair("version", &indyVdr::version));
  fMap.insert(std::make_tuple("setConfig", &indyVdr::setConfig));
  fMap.insert(std::make_tuple("setDefaultLogger", &indyVdr::setDefaultLogger));
  fMap.insert(
      std::make_tuple("setProtocolVersion", &indyVdr::setProtocolVersion));
  fMap.insert(std::make_tuple("setSocksProxy", &indyVdr::setSocksProxy));

  fMap.insert(std::make_tuple("buildAcceptanceMechanismsRequest",
                              &indyVdr::buildAcceptanceMechanismsRequest));
  fMap.insert(std::make_tuple("buildGetAcceptanceMechanismsRequest",
                              &indyVdr::buildGetAcceptanceMechanismsRequest));
  fMap.insert(
      std::make_tuple("buildAttribRequest", &indyVdr::buildAttribRequest));
  fMap.insert(std::make_tuple("buildGetAttribRequest",
                              &indyVdr::buildGetAttribRequest));
  fMap.insert(
      std::make_tuple("buildCredDefRequest", &indyVdr::buildCredDefRequest));
  fMap.insert(std::make_tuple("buildGetCredDefRequest",
                              &indyVdr::buildGetCredDefRequest));
  fMap.insert(std::make_tuple("buildGetRevocRegDefRequest",
                              &indyVdr::buildGetRevocRegDefRequest));
  fMap.insert(std::make_tuple("buildGetRevocRegRequest",
                              &indyVdr::buildGetRevocRegRequest));
  fMap.insert(std::make_tuple("buildGetRevocRegDeltaRequest",
                              &indyVdr::buildGetRevocRegDeltaRequest));
  fMap.insert(std::make_tuple("buildRevocRegDefRequest",
                              &indyVdr::buildRevocRegDefRequest));
  fMap.insert(
      std::make_tuple("buildCustomRequest", &indyVdr::buildCustomRequest));
  fMap.insert(
      std::make_tuple("buildDisableAllTxnAuthorAgreementsRequest",
                      &indyVdr::buildDisableAllTxnAuthorAgreementsRequest));
  fMap.insert(
      std::make_tuple("buildGetNymRequest", &indyVdr::buildGetNymRequest));
  fMap.insert(std::make_tuple("buildGetSchemaRequest",
                              &indyVdr::buildGetSchemaRequest));
  fMap.insert(std::make_tuple("buildGetTxnAuthorAgreementRequest",
                              &indyVdr::buildGetTxnAuthorAgreementRequest));
  fMap.insert(
      std::make_tuple("buildGetTxnRequest", &indyVdr::buildGetTxnRequest));
  fMap.insert(std::make_tuple("buildGetValidatorInfoRequest",
                              &indyVdr::buildGetValidatorInfoRequest));
  fMap.insert(std::make_tuple("buildNymRequest", &indyVdr::buildNymRequest));
  fMap.insert(std::make_tuple("buildRevocRegEntryRequest",
                              &indyVdr::buildRevocRegEntryRequest));
  fMap.insert(
      std::make_tuple("buildSchemaRequest", &indyVdr::buildSchemaRequest));
  fMap.insert(std::make_tuple("buildTxnAuthorAgreementRequest",
                              &indyVdr::buildTxnAuthorAgreementRequest));

  fMap.insert(std::make_tuple("poolCreate", &indyVdr::poolCreate));
  fMap.insert(std::make_tuple("poolRefresh", &indyVdr::poolRefresh));
  fMap.insert(std::make_tuple("poolGetStatus", &indyVdr::poolGetStatus));
  fMap.insert(
      std::make_tuple("poolGetTransactions", &indyVdr::poolGetTransactions));
  fMap.insert(std::make_tuple("poolGetVerifiers", &indyVdr::poolGetVerifiers));
  fMap.insert(std::make_tuple("poolSubmitAction", &indyVdr::poolSubmitAction));
  fMap.insert(
      std::make_tuple("poolSubmitRequest", &indyVdr::poolSubmitRequest));
  fMap.insert(std::make_tuple("poolClose", &indyVdr::poolClose));

  fMap.insert(
      std::make_tuple("requestSetEndorser", &indyVdr::requestSetEndorser));
  fMap.insert(std::make_tuple("requestSetMultiSignature",
                              &indyVdr::requestSetMultiSignature));
  fMap.insert(
      std::make_tuple("requestSetSignature", &indyVdr::requestSetSignature));
  fMap.insert(
      std::make_tuple("requestSetTxnAuthorAgreementAcceptance",
                      &indyVdr::requestSetTxnAuthorAgreementAcceptance));
  fMap.insert(std::make_tuple("requestFree", &indyVdr::requestFree));

  fMap.insert(std::make_tuple("prepareTxnAuthorAgreementAcceptance",
                              &indyVdr::prepareTxnAuthorAgreementAcceptance));
  fMap.insert(std::make_tuple("requestGetBody", &indyVdr::requestGetBody));
  fMap.insert(std::make_tuple("requestGetSignatureInput",
                              &indyVdr::requestGetSignatureInput));

  return fMap;
}

jsi::Function TurboModuleHostObject::call(jsi::Runtime &rt, const char *name,
                                          Cb cb) {
  return jsi::Function::createFromHostFunction(
      rt, jsi::PropNameID::forAscii(rt, name), 1,
      [this, cb](jsi::Runtime &rt, const jsi::Value &thisValue,
                 const jsi::Value *arguments, size_t count) -> jsi::Value {
        const jsi::Value *val = &arguments[0];
        turboModuleUtility::assertValueIsObject(rt, val);
        return (*cb)(rt, val->getObject(rt));
      });
};

std::vector<jsi::PropNameID>
TurboModuleHostObject::getPropertyNames(jsi::Runtime &rt) {
  auto fMap = TurboModuleHostObject::functionMapping(rt);
  std::vector<jsi::PropNameID> result;
  for (FunctionMap::iterator it = fMap.begin(); it != fMap.end(); ++it) {
    result.push_back(jsi::PropNameID::forUtf8(rt, it->first));
  }

  return result;
}

jsi::Value TurboModuleHostObject::get(jsi::Runtime &rt,
                                      const jsi::PropNameID &propNameId) {
  auto propName = propNameId.utf8(rt);
  auto fMap = TurboModuleHostObject::functionMapping(rt);
  for (FunctionMap::iterator it = fMap.begin(); it != fMap.end(); ++it) {
    if (it->first == propName) {
      return TurboModuleHostObject::call(rt, it->first, it->second);
    }
  }

  /*
   * https://overreacted.io/why-do-react-elements-have-typeof-property/
   *
   * This is a special React key on the object that `React.createElement()`
   * returns.
   *
   * This function is called under-the-hood to see if this React element is
   * renderable.
   *
   * When we return undefined, instead of `Symbol.for('react.element'), we tell
   * React that this element is not renderable.
   *
   */
  if (propName == "$$typeof") {
    return jsi::Value::undefined();
  }

  throw jsi::JSError(rt, "Function: " + propName + " is not defined");
}
