"""Methods for generating and working with pool ledger requests."""

from ctypes import byref, c_int32, c_int64, c_uint64
from datetime import datetime, date
from enum import IntEnum
from typing import Optional, Union

from .bindings import RequestHandle, do_call, encode_json, encode_str, lib_string
from .request import Request


class LedgerType(IntEnum):
    """An enum representing the sub-ledger indexes."""

    POOL = 0
    DOMAIN = 1
    CONFIG = 2

    @classmethod
    def from_value(cls, val: [int, str, "LedgerType"]) -> "LedgerType":
        """Initialize a `LedgerType` from an integer or string."""
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
    """
    Builds a SET_TXN_AUTHR_AGRMT_AML request.

    Request to add a new list of acceptance mechanisms for transaction author
    agreement. Acceptance Mechanism is a description of the ways how the user may
    accept a transaction author agreement.

    Args:
        submitter_did: Identifier (DID) of the transaction author as a base58-encoded
            string.
        aml: a set of new acceptance mechanisms:
            {
                "<acceptance mechanism label 1>": { description 1},
                "<acceptance mechanism label 2>": { description 2},
                ...
            }
        version: The version of the new acceptance mechanisms. (Note: unique on the
            Ledger)
        aml_context: (Optional) common context information about acceptance mechanisms
            (may be a URL to external resource)
    """
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
    """
    Builds a DISABLE_ALL_TXN_AUTHR_AGRMTS request.

    Used to disable all Transaction Author Agreements on the ledger.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string.
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    do_call(
        "indy_vdr_build_disable_all_txn_author_agreements_request", did_p, byref(handle)
    )
    return Request(handle)


def build_get_acceptance_mechanisms_request(
    submitter_did: str = None, timestamp: int = None, version: str = None
) -> Request:
    """
    Builds a GET_TXN_AUTHR_AGRMT_AML request.

    Request to get a list of acceptance mechanisms from the ledger valid for specified
    time, or the latest one.

    Args:
        submitter_did: (Optional) DID of the read request sender (if not provided, then
            the default Libindy DID will be used)
        timestamp: (Optional) time to get an active acceptance mechanisms. The latest
            one will be returned for the empty timestamp
        version: (Optional) version of acceptance mechanisms

    NOTE: timestamp and version cannot be specified together.
    """

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
    """
    Builds a GET_NYM request to get information about a DID (NYM).

    Args:
        submitter_did: (Optional) DID of the read request sender (if not provided
            then the default Libindy DID will be used)
        target_did: Target DID as base58-encoded string for 16 or 32 bit DID value
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    dest_p = encode_str(dest)
    do_call("indy_vdr_build_get_nym_request", did_p, dest_p, byref(handle))
    return Request(handle)


def build_get_txn_author_agreement_request(
    submitter_did: str = None, data: Union[bytes, str, dict] = None
) -> Request:
    """
    Builds a GET_TXN_AUTHR_AGRMT request.

    Used to get a specific Transaction Author Agreement from the ledger.

    Args:
        submitter_did: (Optional) DID of the read request sender (if not provided
            then the default Libindy DID will be used).
        data: (Optional) specifies conditions for getting a specific TAA
            Contains 3 mutually exclusive optional fields:
            {
                hash: Optional<str> - hash of requested TAA,
                version: Optional<str> - version of requested TAA.
                timestamp: Optional<i64> - ledger will return TAA valid at requested
                    timestamp.
            }
            Null data or empty JSON are acceptable here. In this case, ledger will
            return the latest version of the TAA.
    """
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
    submitter_did: Optional[str],
    ledger_type: Optional[Union[int, str, LedgerType]],
    seq_no: int,
) -> Request:
    """
    Builds a GET_TXN request to get any transaction by its sequence number.

    Args:
        submitter_did: (Optional) DID of the read request sender (if not provided
            then the default Libindy DID will be used)
        ledger_type: (Optional) type of the ledger the requested transaction belongs to
            Pass a `LedgerType` instance for known values
        seq_no: requested transaction sequence number as it's stored on the ledger
    """

    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    ledger_type = (
        LedgerType.from_value(ledger_type)
        if ledger_type is not None
        else LedgerType.DOMAIN
    )
    do_call(
        "indy_vdr_build_get_txn_request",
        did_p,
        c_int32(ledger_type),
        c_int32(seq_no),
        byref(handle),
    )
    return Request(handle)


def build_get_validator_info_request(submitter_did: str) -> Request:
    """
    Builds a GET_VALIDATOR_INFO request.

    Args:
        submitter_did: DID of the request sender
    """
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
    """
    Builds a NYM request to create new DID on the ledger.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string
        target_did: Target DID as base58-encoded string for 16 or 32 bit DID value
        verkey: (Optional) Target identity verification key as base58-encoded string
        alias: (Optional) The NYM's alias.
        role: (Optional) Role of a user NYM record:
            null (common USER)
            TRUSTEE
            STEWARD
            TRUST_ANCHOR
            ENDORSER - equal to TRUST_ANCHOR that will be removed soon
            NETWORK_MONITOR
            empty string to reset role
    """
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
    """
    Builds a TXN_AUTHR_AGRMT request.

    Used to add a new version of the Transaction Author Agreement to the ledger.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string.
        text: (Optional) the content of the TAA. Mandatory in case of adding a new TAA.
            An existing TAA text can not be changed.
            For Indy Node version <= 1.12.0:
                Use empty string to reset TAA on the ledger
            For Indy Node version > 1.12.0:
                Should be omitted in case of updating an existing TAA (setting
                `retirement_ts`)
        version: the version of the TAA (a unique UTF-8 string)
        ratification_ts: (Optional) the date (timestamp) of TAA ratification by
            network government.
            For Indy Node version <= 1.12.0:
                Must be omitted
            For Indy Node version > 1.12.0:
                Must be specified in case of adding a new TAA
                Can be omitted in case of updating an existing TAA
        retirement_ts: (Optional) the date (timestamp) of TAA retirement
            For Indy Node version <= 1.12.0:
                Must be omitted
            For Indy Node version > 1.12.0:
                Must be omitted in case of adding a new (latest) TAA.
                Should be used for updating (deactivating) non-latest TAA on the
                ledger.

    Note: Use `build_disable_all_txn_author_agreements_request` to disable all TAAs
    on the ledger.
    """
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


def prepare_txn_author_agreement_acceptance(
    text: Optional[str],
    version: Optional[str],
    taa_digest: Optional[str],
    mechanism: str,
    accepted_time: int = None,
) -> str:
    """
    Prepare transaction author agreement acceptance data.

    This function generates transaction author agreement data which can be appended
    to a specific request using `Request.set_txn_author_agreement_acceptance`, which
    must be used if there is any transaction author agreement set on the ledger.

    This function may calculate hash by itself or consume it as a parameter. If all
    text, version and taa_digest parameters are specified, an integrity check will be
    performed on them.

    Args:
        text: (Optional) raw TAA text from the ledger
        version: (Optional) raw TAA version from the ledger
        taa_digest: (Optional) digest of the TAA text and version.
            Digest is sha256 hash calculated on the concatenated strings:
            `version || text`. This parameter is required if text and version
            parameters are omitted
        mechanism: mechanism from the ledger Acceptance Mechanisms List indicating how
            the user has accepted the TAA
        time: UTC timestamp representing when user has accepted the TAA. Note that the
            time portion will be discarded to protect privacy
    """

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
        "indy_vdr_prepare_txn_author_agreement_acceptance",
        text_p,
        version_p,
        taa_digest_p,
        mechanism_p,
        c_uint64(accepted_time),
        byref(result),
    )
    return result.value.decode("utf-8")
