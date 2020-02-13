import asyncio
from ctypes import CDLL, CFUNCTYPE, byref, c_char_p, c_size_t, c_void_p, c_int32
from ctypes.util import find_library
import json
import logging
from typing import Optional, Sequence, Union

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
        get_library().indy_vdr_string_free(self)


def get_library() -> CDLL:
    global LIB
    if LIB is None:
        LIB = _load_library("indy_vdr")
        do_call("indy_vdr_set_default_logger")
    return LIB


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
            return
        loop.call_soon_threadsafe(lambda: _fulfill_future(fut, exc, result))

    res = cb_type(_cb)
    return res


def do_call(fn_name, *args):
    lib_fn = getattr(get_library(), fn_name)
    result = lib_fn(*args)
    if result:
        raise get_current_error(True)


def do_call_async(fn_name, *args, return_type=None):
    lib_fn = getattr(get_library(), fn_name)
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
    result = lib_fn(*args, res)
    if result:
        # callback will not be executed
        fut.set_exception(get_current_error())
    else:
        # keep a reference to the callback function to avoid it being freed
        CALLBACKS[fut] = (loop, res)
    return fut


def encode_json(arg) -> c_char_p:
    return encode_str(json.dumps(arg))


def encode_str(arg: Optional[Union[str, bytes]]) -> c_char_p:
    if arg is None:
        return None
    if isinstance(arg, bytes):
        return c_char_p(arg)
    return c_char_p(arg.encode("utf-8"))


def get_current_error(expect: bool = False) -> VdrError:
    err_json = lib_string()
    if not get_library().indy_vdr_get_current_error(byref(err_json)):
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
    lib = get_library()
    lib.indy_vdr_version.restype = c_void_p
    return lib_string(lib.indy_vdr_version()).value.decode("ascii")


def pool_create_from_genesis_file(path: Union[str, bytes]) -> PoolHandle:
    handle = PoolHandle()
    path_p = encode_str(path)
    do_call("indy_vdr_pool_create_from_genesis_file", path_p, byref(handle))
    return handle


def pool_get_status(pool_handle: PoolHandle) -> asyncio.Future:
    return do_call_async(
        "indy_vdr_pool_get_status", pool_handle, return_type=lib_string
    )


def pool_refresh(pool_handle: PoolHandle) -> asyncio.Future:
    return do_call_async("indy_vdr_pool_refresh", pool_handle)


def pool_submit_action(
    pool_handle: PoolHandle,
    request_handle: RequestHandle,
    nodes: Sequence[str] = None,
    timeout: int = None,
) -> asyncio.Future:
    nodes_p = encode_json(nodes) if nodes else c_void_p()
    timeout = c_int32(-1 if timeout is None else timeout)
    return do_call_async(
        "indy_vdr_pool_submit_action",
        pool_handle,
        request_handle,
        nodes_p,
        timeout,
        return_type=lib_string,
    )


def pool_submit_request(
    pool_handle: PoolHandle, request_handle: RequestHandle
) -> asyncio.Future:
    return do_call_async(
        "indy_vdr_pool_submit_request",
        pool_handle,
        request_handle,
        return_type=lib_string,
    )


def pool_close(pool_handle: PoolHandle):
    do_call("indy_vdr_pool_close", pool_handle)


def pool_get_transactions(pool_handle: PoolHandle) -> asyncio.Future:
    return do_call_async(
        "indy_vdr_pool_get_transactions", pool_handle, return_type=lib_string
    )


def request_free(handle: RequestHandle):
    do_call("indy_vdr_request_free", handle)


def request_get_body(handle: RequestHandle) -> str:
    body = lib_string()
    do_call("indy_vdr_request_get_body", handle, byref(body))
    return body.value.decode("utf-8")


def request_get_signature_input(handle: RequestHandle) -> bytes:
    sig_input = lib_string()
    do_call("indy_vdr_request_get_signature_input", handle, byref(sig_input))
    return sig_input.value


def request_set_endorser(handle: RequestHandle, endorser_did: str):
    endorser_p = encode_str(endorser_did)
    do_call("indy_vdr_request_set_endorser", handle, endorser_p)


def request_set_signature(handle: RequestHandle, signature: bytes):
    sig_len = len(signature)
    do_call("indy_vdr_request_set_signature", handle, signature, sig_len)


def request_set_taa_acceptance(handle: RequestHandle, acceptance: Union[str, dict]):
    acceptance_p = (
        encode_str(acceptance)
        if isinstance(acceptance, str)
        else encode_json(acceptance)
    )
    do_call("indy_vdr_request_set_taa_acceptance", handle, acceptance_p)


def set_config(config: dict):
    # config = {"freshness_threshold": 1}
    do_call("indy_vdr_set_config", encode_json(config))


def set_protocol_version(version: int):
    do_call("indy_vdr_set_protocol_version", c_size_t(version))
