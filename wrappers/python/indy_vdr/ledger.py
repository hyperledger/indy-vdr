from ctypes import byref, c_int32, c_int64, c_uint64
from datetime import datetime, date
from enum import IntEnum
from typing import Optional, Union

from .bindings import RequestHandle, do_call, encode_json, encode_str, lib_string
from .request import Request


class LedgerType(IntEnum):
    POOL = 0
    DOMAIN = 1
    CONFIG = 2

    @classmethod
    def from_value(cls, val: [int, str, "LedgerType"]) -> "LedgerType":
        if isinstance(val, str):
            if val.isdigit():
                return cls(int(val))
            else:
                return cls[val.upper()]
        elif isinstance(val, int):
            return cls(val)
        elif isinstance(val, LedgerType):
            return val
        raise TypeError


def build_acceptance_mechanisms_request(
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
        "indy_vdr_build_acceptance_mechanisms_request",
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
    submitter_did: Optional[str], ledger_type: [int, str, LedgerType], seq_no: int
) -> Request:
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    ledger_type = LedgerType.from_value(ledger_type)
    do_call(
        "indy_vdr_build_get_txn_request",
        did_p,
        c_int32(ledger_type),
        c_int32(seq_no),
        byref(handle),
    )
    return Request(handle)


def build_get_validator_info_request(submitter_did: str) -> Request:
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


def prepare_taa_acceptance(
    text: Optional[str],
    version: Optional[str],
    taa_digest: Optional[str],
    mechanism: str,
    accepted_time: int = None,
) -> str:
    text_p = encode_str(text)
    version_p = encode_str(version)
    taa_digest_p = encode_str(taa_digest)
    mechanism_p = encode_str(mechanism)
    if not accepted_time:
        # rough timestamp
        accepted_time = int(
            datetime.combine(date.today(), datetime.min.time()).timestamp()
        )
    result = lib_string()
    do_call(
        "indy_vdr_prepare_taa_acceptance",
        text_p,
        version_p,
        taa_digest_p,
        mechanism_p,
        c_uint64(accepted_time),
        byref(result),
    )
    return result.value.decode("utf-8")
