#pragma once

#include <jsi/jsi.h>

#include <indyVdr.h>
#include <turboModuleUtility.h>

using namespace facebook;

namespace indyVdr {

jsi::String version(jsi::Runtime &rt);
jsi::String getCurrentError(jsi::Runtime &rt);
void setConfig(jsi::Runtime &rt, const jsi::Object &options);
void setDefaultLogger(jsi::Runtime &rt);
void setProtocolVersion(jsi::Runtime &rt, const jsi::Object &options);
void setSocksProxy(jsi::Runtime &rt, const jsi::Object &options);

jsi::Number buildAcceptanceMechanismsRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetAcceptanceMechanismsRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildAttribRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetAttribRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildCredDefRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetCredDefRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetRevocRegDefRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetRevocRegRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetRevocRegDeltaRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildRevocRegDefRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildCustomRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildDisableAllTxnAuthorAgreementsRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetNymRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetSchemaRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetTxnAuthorAgreementRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetTxnRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetValidatorInfoRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildNymRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildRevocRegEntryRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildSchemaRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildTxnAuthorAgreementRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildRichSchemaRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetRichSchemaObjectByIdRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Number buildGetRichSchemaObjectByMetadataRequest(jsi::Runtime &rt, const jsi::Object &options);

jsi::Number poolCreate(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value poolRefresh(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value poolGetStatus(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value poolGetTransactions(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value poolGetVerifiers(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value poolSubmitAction(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value poolSubmitRequest(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value poolClose(jsi::Runtime &rt, const jsi::Object &options);

jsi::Value requestSetEndorser(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value requestSetMultiSignature(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value requestSetSignature(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value requestSetTxnAuthorAgreementAcceptance(jsi::Runtime &rt, const jsi::Object &options);
jsi::Value requestFree(jsi::Runtime &rt, const jsi::Object &options);

jsi::String prepareTxnAuthorAgreementAcceptance(jsi::Runtime &rt, const jsi::Object &options);
jsi::String request_get_body(jsi::Runtime &rt, const jsi::Object &options);
jsi::String requestGetSignatureInput(jsi::Runtime &rt, const jsi::Object &options);

}
