from ctypes import byref, c_int32, c_int64
from enum import IntEnum
from typing import Optional, Union

from .bindings import RequestHandle, do_call, encode_str, encode_json
from .request import Request


class LedgerType(IntEnum):
    POOL = 0
    DOMAIN = 1
    CONFIG = 2


async def build_acceptance_mechanisms_request(
    submitter_did: str,
    aml: Union[str, bytes, dict],
    version: str,
    aml_context: str = None,
) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    aml_p = encode_str(aml) if isinstance(aml, (str, bytes)) else encode_json(aml)
    version_p = encode_str(version)
    aml_context_p = encode_str(aml_context)
    do_call(
        "indy_vdr_build_get_acceptance_mechanisms_request",
        did_p,
        aml_p,
        version_p,
        aml_context_p,
        byref(handle),
    )
    return Request(handle)


def build_custom_request(body: Union[str, bytes, dict]) -> Request:
    handle = RequestHandle()
    body_p = encode_str(body) if isinstance(body, (str, bytes)) else encode_json(body)
    do_call("indy_vdr_build_custom_request", body_p, byref(handle))
    return Request(handle)


def build_disable_all_txn_author_agreements_request(submitter_did: str) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    do_call(
        "indy_vdr_build_disable_all_txn_author_agreements_request", did_p, byref(handle)
    )
    return Request(handle)


def build_get_acceptance_mechanisms_request(
    submitter_did: str = None, timestamp: int = None, version: str = None
) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    timestamp_c = c_int64(timestamp if timestamp is not None else -1)
    version_p = encode_str(version)
    do_call(
        "indy_vdr_build_get_acceptance_mechanisms_request",
        did_p,
        timestamp_c,
        version_p,
        byref(handle),
    )
    return Request(handle)


def build_get_nym_request(submitter_did: Optional[str], dest: str) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    dest_p = encode_str(dest)
    do_call("indy_vdr_build_get_nym_request", did_p, dest_p, byref(handle))
    return Request(handle)


def build_get_txn_author_agreement_request(
    submitter_did: str = None, data: Union[bytes, str, dict] = None
) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    data_p = (
        encode_str(data)
        if isinstance(data, (str, bytes)) or data is None
        else encode_json(data)
    )
    do_call(
        "indy_vdr_build_get_txn_author_agreement_request", did_p, data_p, byref(handle)
    )
    return Request(handle)


def build_get_txn_request(
    submitter_did: Optional[str], ledger_type: int, seq_no: int
) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    do_call(
        "indy_vdr_build_get_txn_request",
        did_p,
        c_int32(ledger_type),
        c_int32(seq_no),
        byref(handle),
    )
    return Request(handle)


def build_get_validator_info_request(submitter_did: str) -> RequestHandle:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    do_call("indy_vdr_build_get_validator_info_request", did_p, byref(handle))
    return Request(handle)


def build_nym_request(
    submitter_did: str,
    dest: str,
    verkey: str = None,
    alias: str = None,
    role: str = None,
) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    dest_p = encode_str(dest)
    verkey_p = encode_str(verkey) if verkey else None
    alias_p = encode_str(alias) if alias else None
    role_p = encode_str(role) if role else None
    do_call(
        "indy_vdr_build_nym_request",
        did_p,
        dest_p,
        verkey_p,
        alias_p,
        role_p,
        byref(handle),
    )
    return Request(handle)


def build_txn_author_agreement_request(
    submitter_did: str,
    text: Optional[str],
    version: str,
    ratification_ts: int = None,
    retirement_ts: int = None,
) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    text_p = encode_str(text)
    version_p = encode_str(version)
    ratification_ts_c = c_int64(ratification_ts if ratification_ts is not None else -1)
    retirement_ts_c = c_int64(retirement_ts if retirement_ts is not None else -1)
    do_call(
        "indy_vdr_build_txn_author_agreement_request",
        did_p,
        text_p,
        version_p,
        ratification_ts_c,
        retirement_ts_c,
        byref(handle),
    )
    return Request(handle)
