#pragma once

#include <jsi/jsi.h>

#include <include/libindy_vdr.h>
#include <turboModuleUtility.h>

using namespace facebook;

namespace indyVdr {

jsi::Value version(jsi::Runtime &rt, jsi::Object options);
jsi::Value getCurrentError(jsi::Runtime &rt);
jsi::Value setConfig(jsi::Runtime &rt, jsi::Object options);
jsi::Value setDefaultLogger(jsi::Runtime &rt, jsi::Object options);
jsi::Value setProtocolVersion(jsi::Runtime &rt, jsi::Object options);
jsi::Value setSocksProxy(jsi::Runtime &rt, jsi::Object options);

jsi::Value buildAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                            jsi::Object options);
jsi::Value buildGetAcceptanceMechanismsRequest(jsi::Runtime &rt,
                                               jsi::Object options);
jsi::Value buildAttribRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetAttribRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildCredDefRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetCredDefRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetRevocRegRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetRevocRegDeltaRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildRevocRegDefRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildCustomRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildDisableAllTxnAuthorAgreementsRequest(jsi::Runtime &rt,
                                                     jsi::Object options);
jsi::Value buildGetNymRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetSchemaRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetTxnAuthorAgreementRequest(jsi::Runtime &rt,
                                             jsi::Object options);
jsi::Value buildGetTxnRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildGetValidatorInfoRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildNymRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildRevocRegEntryRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildSchemaRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value buildTxnAuthorAgreementRequest(jsi::Runtime &rt,
                                          jsi::Object options);

jsi::Value poolCreate(jsi::Runtime &rt, jsi::Object options);
jsi::Value poolRefresh(jsi::Runtime &rt, jsi::Object options);
jsi::Value poolGetStatus(jsi::Runtime &rt, jsi::Object options);
jsi::Value poolGetTransactions(jsi::Runtime &rt, jsi::Object options);
jsi::Value poolGetVerifiers(jsi::Runtime &rt, jsi::Object options);
jsi::Value poolSubmitAction(jsi::Runtime &rt, jsi::Object options);
jsi::Value poolSubmitRequest(jsi::Runtime &rt, jsi::Object options);
jsi::Value poolClose(jsi::Runtime &rt, jsi::Object options);

jsi::Value requestSetEndorser(jsi::Runtime &rt, jsi::Object options);
jsi::Value requestSetMultiSignature(jsi::Runtime &rt, jsi::Object options);
jsi::Value requestSetSignature(jsi::Runtime &rt, jsi::Object options);
jsi::Value requestSetTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
                                                  jsi::Object options);
jsi::Value requestFree(jsi::Runtime &rt, jsi::Object options);

jsi::Value prepareTxnAuthorAgreementAcceptance(jsi::Runtime &rt,
                                               jsi::Object options);
jsi::Value requestGetBody(jsi::Runtime &rt, jsi::Object options);
jsi::Value requestGetSignatureInput(jsi::Runtime &rt, jsi::Object options);

} // namespace indyVdr
