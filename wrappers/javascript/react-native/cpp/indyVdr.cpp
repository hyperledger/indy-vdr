#include <indyVdr.h>

namespace indyVdr {

jsi::Value getVersion(jsi::Runtime &rt, jsi::Object options) {
  const char *version = indy_vdr_get_version();
  jsi::Object object = jsi::Object(rt);
  object.setProperty(rt, "version", version);
  return object;
}

} // namespace indyVdr
