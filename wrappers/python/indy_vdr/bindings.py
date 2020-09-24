"""Low-level interaction with the indy_vdr library."""

import asyncio
import json
import logging
import os
import sys
from ctypes import CDLL, CFUNCTYPE, byref, c_char_p, c_size_t, c_void_p, c_int32
from ctypes.util import find_library
from typing import Optional, Sequence, Union

from .error import VdrError, VdrErrorCode


CALLBACKS = {}
LIB: CDLL = None
LOGGER = logging.getLogger(__name__)


class PoolHandle(c_size_t):
    """Index of an active Pool instance."""

    def __repr__(self) -> str:
        """Format pool handle as a string."""
        return f"{self.__class__.__name__}({self.value})"


class RequestHandle(c_size_t):
    """Index of an active Request instance."""

    def __repr__(self) -> str:
        """Format request handle as a string."""
        return f"{self.__class__.__name__}({self.value})"


class lib_string(c_char_p):
    """A string allocated by the library."""

    @classmethod
    def from_param(cls):
        """Returns the type ctypes should use for loading the result."""
        return c_void_p

    def __bytes__(self):
        """Convert to bytes."""
        return self.value

    def __str__(self):
        """Convert to str."""
        return self.value.decode("utf-8")

    def __del__(self):
        """Call the string destructor when this instance is released."""
        get_library().indy_vdr_string_free(self)


def get_library() -> CDLL:
    """Return the CDLL instance, loading it if necessary."""
    global LIB
    if LIB is None:
        LIB = _load_library("indy_vdr")
        do_call("indy_vdr_set_default_logger")
    return LIB


def _load_library(lib_name: str) -> CDLL:
    """Load the CDLL library.

    The python module directory is searched first, followed by the usual
    library resolution for the current system.
    """
    lib_prefix_mapping = {"win32": ""}
    lib_suffix_mapping = {"darwin": ".dylib", "win32": ".dll"}
    try:
        os_name = sys.platform
        lib_prefix = lib_prefix_mapping.get(os_name, "lib")
        lib_suffix = lib_suffix_mapping.get(os_name, ".so")
        lib_path = os.path.join(
            os.path.dirname(__file__), f"{lib_prefix}{lib_name}{lib_suffix}"
        )
        return CDLL(lib_path)
    except KeyError:
        LOGGER.debug("Unknown platform for shared library")
    except OSError:
        LOGGER.warning("Library not loaded from python package")

    lib_path = find_library(lib_name)
    if not lib_path:
        raise VdrError(VdrErrorCode.WRAPPER, f"Error loading library: {lib_name}")
    try:
        return CDLL(lib_path)
    except OSError as e:
        raise VdrError(
            VdrErrorCode.WRAPPER, f"Error loading library: {lib_name}"
        ) from e


def _fulfill_future(fut: asyncio.Future, result, err: Exception = None):
    """Resolve a callback future given the result and exception, if any."""
    if fut.cancelled():
        LOGGER.debug("callback previously cancelled")
    elif err:
        fut.set_exception(err)
    else:
        fut.set_result(result)


def _create_callback(cb_type: CFUNCTYPE, fut: asyncio.Future, post_process=None):
    """Create a callback to handle the response from an async library method."""

    def _cb(id: int, err: int, result=None):
        """Callback function passed to the CFUNCTYPE for invocation."""
        if post_process:
            result = post_process(result)
        exc = get_current_error() if err else None
        try:
            (loop, _cb) = CALLBACKS.pop(fut)
        except KeyError:
            LOGGER.debug("callback already fulfilled")
            return
        loop.call_soon_threadsafe(lambda: _fulfill_future(fut, result, exc))

    res = cb_type(_cb)
    return res


def do_call(fn_name, *args):
    """Perform a synchronous library function call."""
    lib_fn = getattr(get_library(), fn_name)
    result = lib_fn(*args)
    if result:
        raise get_current_error(True)


def do_call_async(fn_name, *args, return_type=None, post_process=None):
    """Perform an asynchronous library function call."""
    lib_fn = getattr(get_library(), fn_name)
    loop = asyncio.get_event_loop()
    fut = loop.create_future()
    cf_args = [None, c_size_t, c_size_t]
    if return_type:
        cf_args.append(return_type)
    cb_type = CFUNCTYPE(*cf_args)  # could be cached
    cb_res = _create_callback(cb_type, fut, post_process)
    # keep a reference to the callback function to avoid it being freed
    CALLBACKS[fut] = (loop, cb_res)
    result = lib_fn(*args, cb_res, c_size_t(0))  # not making use of callback ID
    if result:
        # callback will not be executed
        del CALLBACKS[fut]
        fut.set_exception(get_current_error())
    return fut


def encode_json(arg) -> c_char_p:
    """Encode an input argument as JSON."""
    return encode_str(json.dumps(arg))


def encode_str(arg: Optional[Union[str, bytes]]) -> c_char_p:
    """Encode an optional input argument as a string.

    Returns: None if the argument is None, otherwise the value encoded utf-8.
    """
    if arg is None:
        return None
    if isinstance(arg, bytes):
        return c_char_p(arg)
    return c_char_p(arg.encode("utf-8"))


def get_current_error(expect: bool = False) -> VdrError:
    """Get the error result from the previous failed API method.

    Args:
        expect: Return a default error message if none is found
    """
    err_json = lib_string()
    if not get_library().indy_vdr_get_current_error(byref(err_json)):
        try:
            msg = json.loads(err_json.value)
        except json.JSONDecodeError:
            LOGGER.warning("JSON decode error for indy_vdr_get_current_error")
            msg = None
        if msg and "message" in msg and "code" in msg:
            return VdrError(VdrErrorCode(msg["code"]), msg["message"], msg.get("extra"))
        if not expect:
            return None
    return VdrError(VdrError.WRAPPER, "Unknown error")


def pool_create(params: Union[str, bytes, dict]) -> PoolHandle:
    """Create a new pool instance.

    Args:
        params: A JSON-encoded str or bytes instance or a dict representing the
            pool creation parameters. See `pool.open_pool` for the parameters.
    """
    handle = PoolHandle()
    params_p = (
        encode_str(params) if isinstance(params, (str, bytes)) else encode_json(params)
    )
    do_call("indy_vdr_pool_create", params_p, byref(handle))
    return handle


def pool_get_status(pool_handle: PoolHandle) -> asyncio.Future:
    """Get the status of an opened pool instance."""
    return do_call_async(
        "indy_vdr_pool_get_status",
        pool_handle,
        return_type=lib_string,
        post_process=str,
    )


def pool_refresh(pool_handle: PoolHandle) -> asyncio.Future:
    """Fetch the latest transactions for the ledger's verifier pool."""
    return do_call_async("indy_vdr_pool_refresh", pool_handle)


def pool_submit_action(
    pool_handle: PoolHandle,
    request_handle: RequestHandle,
    node_aliases: Sequence[str] = None,
    timeout: int = None,
) -> asyncio.Future:
    """Publishes a prepared pool action request message to the validator pool."""
    nodes_p = encode_json(node_aliases) if node_aliases else c_void_p()
    timeout = c_int32(-1 if timeout is None else timeout)
    return do_call_async(
        "indy_vdr_pool_submit_action",
        pool_handle,
        request_handle,
        nodes_p,
        timeout,
        return_type=lib_string,
        post_process=str,
    )


def pool_submit_request(
    pool_handle: PoolHandle, request_handle: RequestHandle
) -> asyncio.Future:
    """Publishes a prepared request message to the validator pool."""
    return do_call_async(
        "indy_vdr_pool_submit_request",
        pool_handle,
        request_handle,
        return_type=lib_string,
        post_process=str,
    )


def pool_close(pool_handle: PoolHandle):
    """Close and free a pool instance."""
    do_call("indy_vdr_pool_close", pool_handle)


def pool_get_transactions(pool_handle: PoolHandle) -> asyncio.Future:
    """Fetch the transactions for an opened pool instance."""
    return do_call_async(
        "indy_vdr_pool_get_transactions",
        pool_handle,
        return_type=lib_string,
        post_process=str,
    )


def pool_get_verifiers(pool_handle: PoolHandle) -> asyncio.Future:
    """Fetch the set of active verifiers for an opened pool instance."""
    return do_call_async(
        "indy_vdr_pool_get_verifiers",
        pool_handle,
        return_type=lib_string,
        post_process=str,
    )


def request_free(handle: RequestHandle):
    """Manually free a prepared request which won't be submitted."""
    do_call("indy_vdr_request_free", handle)


def request_get_body(handle: RequestHandle) -> str:
    """Get the canonical signature input for a prepared request."""
    body = lib_string()
    do_call("indy_vdr_request_get_body", handle, byref(body))
    return body.value.decode("utf-8")


def request_get_signature_input(handle: RequestHandle) -> bytes:
    """Get the canonical signature input for a prepared request."""
    sig_input = lib_string()
    do_call("indy_vdr_request_get_signature_input", handle, byref(sig_input))
    return sig_input.value


def request_set_endorser(handle: RequestHandle, endorser_did: str):
    """Set the endorser on a prepared request."""
    endorser_p = encode_str(endorser_did)
    do_call("indy_vdr_request_set_endorser", handle, endorser_p)


def request_set_signature(handle: RequestHandle, signature: bytes):
    """Set the signature on a prepared request."""
    sig_len = len(signature)
    do_call("indy_vdr_request_set_signature", handle, signature, sig_len)


def request_set_txn_author_agreement_acceptance(
    handle: RequestHandle, acceptance: Union[str, dict]
):
    """Set the transaction author agreement acceptance on a prepared request."""
    acceptance_p = (
        encode_str(acceptance)
        if isinstance(acceptance, str)
        else encode_json(acceptance)
    )
    do_call(
        "indy_vdr_request_set_txn_author_agreement_acceptance", handle, acceptance_p
    )


def set_config(config: dict):
    """Set the library configuration."""
    do_call("indy_vdr_set_config", encode_json(config))


def set_protocol_version(version: int):
    """Set the library protocol version."""
    do_call("indy_vdr_set_protocol_version", c_size_t(version))


def version() -> str:
    """Set the version of the installed indy_vdr library."""
    lib = get_library()
    lib.indy_vdr_version.restype = c_void_p
    return lib_string(lib.indy_vdr_version()).value.decode("utf-8")
