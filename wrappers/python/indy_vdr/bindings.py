from ctypes import c_char_p, CDLL, c_size_t, byref
from ctypes.util import find_library
from enum import IntEnum
import json
from typing import Union


class VdrErrorCode(IntEnum):
    SUCCESS = 0
    FAILED = 1
    OTHER = 99


class VdrError(Exception):
    def __init__(self, code: VdrErrorCode, message: str):
        super().__init__(message)
        self.code = code


class PoolHandle(c_size_t):
    pass


class RequestHandle(c_size_t):
    pass


class lib_char_p(c_char_p):
    def __del__(self):
        LIB.indy_vdr_string_free(self)


def _load_library(lib_name: str) -> CDLL:
    found = find_library(lib_name)
    if not found:
        raise VdrError(VdrErrorCode.OTHER, f"Error loading library: {lib_name}")
    return CDLL(name=found)
    # except OSError:


def _do_call(fn, *args):
    result = fn(*args)
    if result:
        err_json = lib_char_p()
        if not LIB.indy_vdr_get_last_error(byref(err_json)):
            try:
                msg = json.loads(err_json.value)
            except json.JSONDecodeError:
                msg = None
            if msg and "message" in msg and "code" in msg:
                raise VdrError(VdrErrorCode(msg["code"]), msg["message"])
        raise VdrError(VdrError.OTHER, "Unknown error")


def _encode_json(arg) -> c_char_p:
    return _encode_str(json.dumps(arg))


def _encode_str(arg: Union[str, bytes]) -> c_char_p:
    if isinstance(arg, bytes):
        return c_char_p(arg)
    return c_char_p(arg.encode("utf-8"))


def build_custom_request(body: Union[str, bytes, dict]) -> RequestHandle:
    handle = c_size_t()
    body_p = _encode_str(body) if isinstance(body, (str, bytes)) else _encode_json(body)
    _do_call(LIB.indy_vdr_build_custom_request, body_p, byref(handle))
    return handle


def request_get_body(handle: RequestHandle) -> str:
    body = lib_char_p()
    _do_call(LIB.indy_vdr_request_get_body, handle, byref(body))
    return body.value.decode("utf-8")


def request_get_signature_input(handle: RequestHandle) -> bytes:
    sig_input = lib_char_p()
    _do_call(LIB.indy_vdr_request_get_signature_input, handle, byref(sig_input))
    return sig_input.value


def request_set_signature(handle: RequestHandle, signature: bytes):
    sig_len = len(signature)
    _do_call(LIB.indy_vdr_request_set_signature, handle, signature, sig_len)


def set_config(config: dict):
    # config = {"freshness_threshold": 1}
    _do_call(LIB.indy_vdr_set_config, _encode_json(config))


def set_protocol_version(version: int):
    _do_call(LIB.indy_vdr_set_protocol_version, c_size_t(version))


LIB = _load_library("indy_vdr")
_do_call(LIB.indy_vdr_set_default_logger)
