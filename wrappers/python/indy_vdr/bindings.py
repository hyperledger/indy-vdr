from ctypes import c_char_p, CDLL, c_size_t, byref
from ctypes.util import find_library
import json
import os
from typing import Union


def _load_library(lib_name: str) -> CDLL:
    found = find_library(lib_name)
    if not found:
        raise Exception(f"Error loading library: {lib_name}")
    return CDLL(name=found)
    # except OSError:


os.environ["RUST_LOG"] = "debug"
lib = _load_library("indy_vdr")


def _do_call(fn, *args):
    result = fn(*args)
    if result:
        raise Exception(result)


def _encode_json(arg) -> c_char_p:
    return _encode_str(json.dumps(arg))


def _encode_str(arg: Union[str, bytes]) -> c_char_p:
    if isinstance(arg, bytes):
        return c_char_p(arg)
    return c_char_p(arg.encode("utf-8"))


class lib_char_p(c_char_p):
    def __del__(self):
        lib.indy_vdr_string_free(self)


def build_custom_request(body: Union[str, bytes, dict]) -> c_size_t:
    handle = c_size_t()
    body_p = _encode_str(body) if isinstance(body, (str, bytes)) else _encode_json(body)
    _do_call(lib.indy_vdr_build_custom_request, body_p, byref(handle))
    return handle


def request_get_body(handle: c_size_t) -> str:
    body = lib_char_p()
    _do_call(lib.indy_vdr_request_get_body, handle, byref(body))
    return body.value.decode("utf-8")


def request_get_signature_input(handle: c_size_t) -> bytes:
    sig_input = lib_char_p()
    _do_call(lib.indy_vdr_request_get_signature_input, handle, byref(sig_input))
    return sig_input.value


def set_config(config: dict):
    # config = {"freshness_threshold": 1}
    _do_call(lib.indy_vdr_set_config, _encode_json(config))


def set_protocol_version(version: int):
    _do_call(lib.indy_vdr_set_protocol_version, c_size_t(version))
