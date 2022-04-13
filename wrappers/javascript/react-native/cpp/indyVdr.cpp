#include <indyVdr.h>

namespace indyVdr {

const char* getVersion() {
  const char* version = ::indy_vdr_version();
  return version;
}

}
