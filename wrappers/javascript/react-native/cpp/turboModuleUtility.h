#pragma once

#include <HostObject.h>
#include <jsi/jsi.h>

using namespace facebook;

namespace turboModuleUtility {

void registerTurboModule(jsi::Runtime &rt);

void assertValueIsObject(jsi::Runtime &rt, const jsi::Value *val);

template <typename T>
T jsiToValue(jsi::Runtime &rt, jsi::Value value, bool optional = false);

void handleError(jsi::Runtime &rt, ErrorCode code);

} // namespace turboModuleUtility
