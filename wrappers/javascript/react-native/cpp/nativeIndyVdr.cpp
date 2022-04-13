#include <nativeIndyVdr.h>

namespace nativeIndyVdr {

jsi::Value getVersion(jsi::Runtime &rt, jsi::Object options) {
  const char* version = indyVdr::getVersion();
  jsi::Object object = jsi::Object(rt);
  object.setProperty(rt, "version", version);
  return object;
}

}
