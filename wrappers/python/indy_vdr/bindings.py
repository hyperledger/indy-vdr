import asyncio
from ctypes import CDLL, CFUNCTYPE, byref, c_char_p, c_size_t, c_void_p, c_int32
from ctypes.util import find_library
import json
import logging
from typing import Sequence, Union

from .error import VdrError, VdrErrorCode


CALLBACKS = {}
LIB: CDLL = None
LOGGER = logging.getLogger(__name__)


class PoolHandle(c_size_t):
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}({self.value})"


class RequestHandle(c_size_t):
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}({self.value})"


class lib_string(c_char_p):
    @classmethod
    def load_c_ptr(cls, value):
        # convert to lib_string, which will call library's string destructor
        inst = cls(value)
        return inst.value.decode("utf-8")

    def __del__(self):
        LIB.indy_vdr_string_free(self)


def _load_library(lib_name: str) -> CDLL:
    found = find_library(lib_name)
    if not found:
        raise VdrError(VdrErrorCode.WRAPPER, f"Error loading library: {lib_name}")
    return CDLL(name=found)
    # except OSError:


def _fulfill_future(fut: asyncio.Future, err: Exception, result):
    if fut.cancelled():
        LOGGER.debug("callback previously cancelled")
    elif err:
        fut.set_exception(err)
    else:
        fut.set_result(result)


def _create_callback(cb_type: CFUNCTYPE, fut: asyncio.Future, post_proc=None):
    def _cb(err: int, result=None):
        if post_proc:
            result = post_proc(result)
        exc = get_current_error() if err else None
        try:
            (loop, _cb) = CALLBACKS.pop(fut)
        except KeyError:
            LOGGER.debug("callback already fulfilled")
        loop.call_soon_threadsafe(lambda: _fulfill_future(fut, exc, result))

    res = cb_type(_cb)
    return res


def _do_call(fn, *args):
    result = fn(*args)
    if result:
        raise get_current_error(True)


def _do_call_async(fn, *args, return_type=None):
    loop = asyncio.get_event_loop()
    fut = loop.create_future()
    cf_args = [None, c_size_t]
    post_proc = None
    if return_type:
        if hasattr(return_type, "load_c_ptr"):
            post_proc = return_type.load_c_ptr
            return_type = c_void_p
        cf_args.append(return_type)
    cb_type = CFUNCTYPE(*cf_args)  # could be cached
    res = _create_callback(cb_type, fut, post_proc)
    result = fn(*args, res)
    if result:
        # callback will not be executed
        fut.set_exception(get_current_error())
    else:
        # keep a reference to the callback function to avoid it being freed
        CALLBACKS[fut] = (loop, res)
    return fut


def _encode_json(arg) -> c_char_p:
    return _encode_str(json.dumps(arg))


def _encode_str(arg: Union[str, bytes]) -> c_char_p:
    if isinstance(arg, bytes):
        return c_char_p(arg)
    return c_char_p(arg.encode("utf-8"))


def get_current_error(expect: bool = False) -> VdrError:
    err_json = lib_string()
    if not LIB.indy_vdr_get_current_error(byref(err_json)):
        try:
            msg = json.loads(err_json.value)
        except json.JSONDecodeError:
            msg = None
        if msg and "message" in msg and "code" in msg:
            return VdrError(VdrErrorCode(msg["code"]), msg["message"], msg.get("extra"))
        if not expect:
            return None
    return VdrError(VdrError.WRAPPER, "Unknown error")


def get_version() -> str:
    LIB.indy_vdr_version.restype = c_void_p
    return lib_string(LIB.indy_vdr_version()).value.decode("ascii")


def build_custom_request(body: Union[str, bytes, dict]) -> RequestHandle:
    handle = RequestHandle()
    body_p = _encode_str(body) if isinstance(body, (str, bytes)) else _encode_json(body)
    _do_call(LIB.indy_vdr_build_custom_request, body_p, byref(handle))
    return handle


def build_get_txn_request(
    ledger_type: int, seq_no: int, submitter_did: str = None
) -> RequestHandle:
    handle = RequestHandle()
    did_p = _encode_str(submitter_did) if submitter_did else None
    _do_call(
        LIB.indy_vdr_build_get_txn_request,
        did_p,
        c_int32(ledger_type),
        c_int32(seq_no),
        byref(handle),
    )
    return handle


def build_get_validator_info_request(submitter_did: str) -> RequestHandle:
    handle = RequestHandle()
    did_p = _encode_str(submitter_did)
    _do_call(LIB.indy_vdr_build_get_validator_info_request, did_p, byref(handle))
    return handle


def pool_create_from_genesis_file(path: Union[str, bytes]) -> PoolHandle:
    handle = PoolHandle()
    path_p = _encode_str(path)
    _do_call(LIB.indy_vdr_pool_create_from_genesis_file, path_p, byref(handle))
    return handle


def pool_refresh(pool_handle: PoolHandle) -> asyncio.Future:
    return _do_call_async(LIB.indy_vdr_pool_refresh, pool_handle)


def pool_submit_action(
    pool_handle: PoolHandle,
    request_handle: RequestHandle,
    nodes: Sequence[str] = None,
    timeout: int = None,
) -> asyncio.Future:
    nodes_p = _encode_json(nodes) if nodes else c_void_p()
    timeout = c_int32(-1 if timeout is None else timeout)
    return _do_call_async(
        LIB.indy_vdr_pool_submit_action,
        pool_handle,
        request_handle,
        nodes_p,
        timeout,
        return_type=lib_string,
    )


def pool_submit_request(
    pool_handle: PoolHandle, request_handle: RequestHandle
) -> asyncio.Future:
    return _do_call_async(
        LIB.indy_vdr_pool_submit_request,
        pool_handle,
        request_handle,
        return_type=lib_string,
    )


def pool_close(pool_handle: PoolHandle):
    _do_call(LIB.indy_vdr_pool_close, pool_handle)


def pool_get_transactions(pool_handle: PoolHandle) -> asyncio.Future:
    return _do_call_async(
        LIB.indy_vdr_pool_get_transactions, pool_handle, return_type=lib_string
    )


def request_get_body(handle: RequestHandle) -> str:
    body = lib_string()
    _do_call(LIB.indy_vdr_request_get_body, handle, byref(body))
    return body.value.decode("utf-8")


def request_get_signature_input(handle: RequestHandle) -> bytes:
    sig_input = lib_string()
    _do_call(LIB.indy_vdr_request_get_signature_input, handle, byref(sig_input))
    return sig_input.value


def request_set_signature(handle: RequestHandle, signature: bytes):
    sig_len = len(signature)
    _do_call(LIB.indy_vdr_request_set_signature, handle, signature, sig_len)


def request_free(handle: RequestHandle):
    _do_call(LIB.indy_vdr_request_free, handle)


def set_config(config: dict):
    # config = {"freshness_threshold": 1}
    _do_call(LIB.indy_vdr_set_config, _encode_json(config))


def set_protocol_version(version: int):
    _do_call(LIB.indy_vdr_set_protocol_version, c_size_t(version))


LIB = _load_library("indy_vdr")
_do_call(LIB.indy_vdr_set_default_logger)
