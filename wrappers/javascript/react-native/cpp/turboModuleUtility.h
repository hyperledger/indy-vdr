#pragma once

#include <jsi/jsi.h>

#include <HostObject.h>
#include <libindy_vdr.h>

using namespace facebook;

namespace turboModuleUtility {

void registerTurboModule(jsi::Runtime &rt);

void assertValueIsObject(jsi::Runtime &rt, const jsi::Value *val);

template <typename T>
T jsiToValue(jsi::Runtime &rt, jsi::Value value, const char *name,
             bool optional = false);

void handleError(jsi::Runtime &rt, ErrorCode code);

} // namespace turboModuleUtility
