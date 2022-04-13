#include <turboModuleUtility.h>

namespace turboModuleUtility {

void registerTurboModule(jsi::Runtime &rt) {
  // Create a TurboModuleRustHostObject
  auto instance = std::make_shared<TurboModuleHostObject>(rt);
  // Create a JS equivalent object of the instance
  jsi::Object jsInstance = jsi::Object::createFromHostObject(rt, instance);
  // Register the object on global
  rt.global().setProperty(rt, "_indy_vdr", std::move(jsInstance));
}

void assertValueIsObject(jsi::Runtime &rt, const jsi::Value *val) {
  val->asObject(rt);
}

void handleError(jsi::Runtime &rt, ErrorCode code) {
    int error_code = int(code);

    if (error_code == 0) return;

    const char *error_message;
    indy_vdr_get_current_error(&error_message);

    throw jsi::JSError(rt, error_message);
};

template <>
uint8_t jsiToValue<uint8_t>(jsi::Runtime &rt, jsi::Value value, bool optional) {
  if (value.isNumber())
    return value.asNumber();

  throw jsi::JSError(rt, "Value is not of type number");
}

template <>
std::string jsiToValue<std::string>(jsi::Runtime &rt, jsi::Value value,
                                    bool optional) {
  if ((value.isNull() || value.isUndefined()) && optional)
    return std::string();

  if (value.isString())
    return value.asString(rt).utf8(rt);

  throw jsi::JSError(rt, "Value is not of type string");
}

template <>
int64_t jsiToValue<int64_t>(jsi::Runtime &rt, jsi::Value value, bool optional) {
  if (value.isNumber())
    return value.asNumber();

  throw jsi::JSError(rt, "Value is not of type number");
}

template <>
uint64_t jsiToValue<uint64_t>(jsi::Runtime &rt, jsi::Value value,
                              bool optional) {
  if (value.isNumber())
    return value.asNumber();

  throw jsi::JSError(rt, "Value is not of type number");
}

template <>
int32_t jsiToValue<int32_t>(jsi::Runtime &rt, jsi::Value value, bool optional) {
  if (value.isNumber())
    return value.asNumber();

  throw jsi::JSError(rt, "Value is not of type number");
}

template <>
uint32_t jsiToValue<uint32_t>(jsi::Runtime &rt, jsi::Value value,
                              bool optional) {
  if (value.isNumber())
    return value.asNumber();

  throw jsi::JSError(rt, "Value is not of type number");
}


template <>
std::vector<int32_t> jsiToValue<std::vector<int32_t>>(jsi::Runtime &rt,
                                                      jsi::Value value,
                                                      bool optional) {
  if (value.isObject() && value.asObject(rt).isArray(rt)) {
    std::vector<int32_t> vec = {};
    jsi::Array arr = value.asObject(rt).asArray(rt);
    size_t length = arr.length(rt);
    for (int i = 0; i < length; i++) {
      jsi::Value element = arr.getValueAtIndex(rt, i);
      if (element.isNumber()) {
        vec.push_back(element.asNumber());
      } else {
        throw jsi::JSError(rt, "Value in array not of type int64_t");
      }
    }
    return vec;
  }
  if (optional)
    return {};

  throw jsi::JSError(rt, "Value is not of type []");
}

} // namespace turboModuleUtility
