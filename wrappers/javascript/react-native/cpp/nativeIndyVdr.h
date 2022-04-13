#pragma once 

#include <jsi/jsi.h>

#include <indyVdr.h>
#include <turboModuleUtility.h>

using namespace facebook;

namespace nativeIndyVdr {

jsi::Value getVersion(jsi::Runtime &rt, jsi::Object options);

}
