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


def build_attrib_request(
    submitter_did: Optional[str],
    target_did: str,
    xhash: Optional[str],
    raw: Optional[str],
    enc: Optional[str],
) -> str:
    """
    Builds an ATTRIB request.

    Request to add attribute to a NYM record.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string.
        target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
        xhash: (Optional) Hash of attribute data.
        raw: (Optional) JSON, where key is attribute name and value is attribute value.
        enc: (Optional) Encrypted value attribute data.
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    target_p = encode_str(target_did)
    raw_p = encode_str(raw)
    hash_p = encode_str(xhash)
    enc_p = encode_str(enc)
    do_call(
        "indy_vdr_build_attrib_request",
        did_p,
        target_p,
        hash_p,
        raw_p,
        enc_p,
        byref(handle),
    )
    return Request(handle)


def build_cred_def_request(
    submitter_did: str, cred_def: Union[bytes, str, dict]
) -> Request:
    """
    Builds a CRED_DEF request to to add a credential definition to the ledger.

    In particular, this publishes the public key that the issuer created for
    issuing credentials against a particular schema.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string
        cred_def: Credential definition.
            ```jsonc
            {
                "id": "<credential definition identifier>",
                "schemaId": "<schema identifier>",
                "type": "CL",
                    // type of the credential definition. CL is currently
                    // the only supported type
                "tag": "",
                    // allows to distinguish between credential definitions
                    // for the same issuer and schema
                "value": /* Dictionary with Credential Definition's data: */ {
                    "primary": "<primary credential public key>",
                    "revocation": /* Optional */ "<revocation credential public key>"
                },
                ver: Version of the CredDef json
            }```
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    cred_def_p = (
        encode_str(cred_def)
        if isinstance(cred_def, (str, bytes))
        else encode_json(cred_def)
    )
    do_call("indy_vdr_build_cred_def_request", did_p, cred_def_p, byref(handle))
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


def build_get_attrib_request(
    submitter_did: Optional[str],
    target_did: str,
    raw: Optional[str],
    xhash: Optional[str],
    enc: Optional[str],
) -> str:
    """
    Builds a GET_ATTRIB request.

    Request to get information about an Attribute for the specified DID.

    Args:
        submitter_did: (Optional) DID of the read request sender (if not provided, then
            the default Libindy DID will be used).
        target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
        xhash: (Optional) Requested attribute name.
        raw: (Optional) Requested attribute hash.
        enc: (Optional) Requested attribute encrypted value.
    """

    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    target_p = encode_str(target_did)
    raw_p = encode_str(raw)
    hash_p = encode_str(xhash)
    enc_p = encode_str(enc)
    do_call(
        "indy_vdr_build_get_attrib_request",
        did_p,
        target_p,
        raw_p,
        hash_p,
        enc_p,
        byref(handle),
    )
    return Request(handle)


def build_get_cred_def_request(
    submitter_did: Optional[str], cred_def_id: str
) -> Request:
    """
    Builds a GET_CRED_DEF request to fetch a credential definition by ID.

    Args:
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be used
        cred_def_id: ID of the corresponding credential definition on the ledger
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    cred_def_id_p = encode_str(cred_def_id)
    do_call("indy_vdr_build_get_cred_def_request", did_p, cred_def_id_p, byref(handle))
    return Request(handle)


def build_get_nym_request(submitter_did: Optional[str], dest: str) -> Request:
    """
    Builds a GET_NYM request to get information about a DID (NYM).

    Args:
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be use)
        target_did: Target DID as base58-encoded string for 16 or 32 bit DID value
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    dest_p = encode_str(dest)
    do_call("indy_vdr_build_get_nym_request", did_p, dest_p, byref(handle))
    return Request(handle)


def build_get_revoc_reg_def_request(
    submitter_did: Optional[str], revoc_reg_id: str
) -> Request:
    """
    Builds a GET_REVOC_REG_DEF request.

    Request to get the revocation registry definition for a given revocation
    registry ID.

    Args:
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be used.
        revoc_reg_id: ID of the corresponding revocation registry definition
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    rev_id_p = encode_str(revoc_reg_id)
    do_call("indy_vdr_build_get_revoc_reg_def_request", did_p, rev_id_p, byref(handle))
    return Request(handle)


def build_get_revoc_reg_request(
    submitter_did: Optional[str], revoc_reg_id: str, timestamp: int
) -> Request:
    """
    Builds a GET_REVOC_REG request.

    Request to get the accumulated state of the revocation registry by ID. The state
    is defined by the given timestamp.

    Args:
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be used
        revoc_reg_id: ID of the corresponding revocation registry definition
        timestamp: Requested time represented as a total number of seconds since the
            Unix epoch
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    rev_id_p = encode_str(revoc_reg_id)
    timestamp_c = c_int64(timestamp)
    do_call(
        "indy_vdr_build_get_revoc_reg_request",
        did_p,
        rev_id_p,
        timestamp_c,
        byref(handle),
    )
    return Request(handle)


def build_get_revoc_reg_delta_request(
    submitter_did: Optional[str], revoc_reg_id: str, from_ts: Optional[int], to_ts: int
) -> Request:
    """
    Builds a GET_REVOC_REG_DELTA request.

    Request to get the delta of the accumulated state of the revocation registry
    identified by `revoc_reg_id`. The delta is defined by from and to timestamp fields.
    If from is not specified, then the whole state until `to` will be returned.

    Args:
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be used
        revoc_reg_id: ID of the corresponding revocation registry definition
        from_ts: Requested time represented as a total number of seconds from Unix epoch
        to_ts: Requested time represented as a total number of seconds from Unix epoch
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    rev_id_p = encode_str(revoc_reg_id)
    from_c = c_int64(from_ts if from_ts is not None else -1)
    to_c = c_int64(to_ts)
    do_call(
        "indy_vdr_build_get_revoc_reg_delta_request",
        did_p,
        rev_id_p,
        from_c,
        to_c,
        byref(handle),
    )
    return Request(handle)


def build_get_schema_request(submitter_did: Optional[str], schema_id: str) -> Request:
    """
    Builds a GET_SCHEMA request to fetch a credential schema by ID.

    Args:
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be used
        schema_id: ID of the corresponding schema on the ledger
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    schema_id_p = encode_str(schema_id)
    do_call("indy_vdr_build_get_schema_request", did_p, schema_id_p, byref(handle))
    return Request(handle)


def build_get_txn_author_agreement_request(
    submitter_did: str = None, data: Union[bytes, str, dict] = None
) -> Request:
    """
    Builds a GET_TXN_AUTHR_AGRMT request.

    Used to get a specific Transaction Author Agreement from the ledger.

    Args:
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be used
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
        submitter_did: (Optional) DID of the read request sender. If not provided
            then the default Libindy DID will be used
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


def build_revoc_reg_def_request(
    submitter_did: str, revoc_reg_def: Union[bytes, str, dict]
) -> Request:
    """
    Builds a REVOC_REG_DEF request.

    Request to add the definition of revocation registry to an existing
    credential definition.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string
        revoc_reg_def: Revocation Registry data:
            ```jsonc
            {
                "id": "<revocation registry identifier>",
                "revocDefType": "CL_ACCUM",
                    // revocation registry type (only CL_ACCUM is supported for now)
                "tag": "", // Unique descriptive ID of the registry definition
                "credDefId": "<credential definition ID>",
                "value": /* Registry-specific data */ {
                    "issuanceType": "ISSUANCE_BY_DEFAULT",
                        // Type of issuance: ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND
                    "maxCredNum": 10000,
                        // Maximum number of credentials the Registry can serve.
                    "tailsHash": "<sha256 hash of tails file in base58>",
                    "tailsLocation": "<URL or path for the tails file>",
                    "publicKeys": { /* <public_keys> */ } // registry's public keys
                },
                "ver": "<version of revocation registry definition json>"
            }```
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    revoc_reg_def_p = (
        encode_str(revoc_reg_def)
        if isinstance(revoc_reg_def, (str, bytes))
        else encode_json(revoc_reg_def)
    )
    do_call(
        "indy_vdr_build_revoc_reg_def_request", did_p, revoc_reg_def_p, byref(handle)
    )
    return Request(handle)


def build_revoc_reg_entry_request(
    submitter_did: str,
    revoc_reg_def_id: str,
    revoc_reg_def_type: str,
    entry: Union[bytes, str, dict],
) -> Request:
    """
    Builds a REVOC_REG_ENTRY request.

    Request to add the revocation registry entry containing the new accumulator
    value and issued/revoked indices. This is just a delta of indices, not the
    whole list. It can be sent each time a new credential is issued/revoked.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string
        entry: Registry-specific data:
            ```jsonc
            {
                "value": {
                    "prevAccum": "<previous accumulator value>",
                    "accum": "<current accumulator value>",
                    "issued": [], // array<number> - an array of issued indices
                    "revoked": [] // array<number> an array of revoked indices
                },
                "ver": "<version of the revocation registry entry json>"
            }```
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    def_id_p = encode_str(revoc_reg_def_id)
    def_type_p = encode_str(revoc_reg_def_type)
    entry_p = (
        encode_str(entry) if isinstance(entry, (str, bytes)) else encode_json(entry)
    )
    do_call(
        "indy_vdr_build_revoc_reg_entry_request",
        did_p,
        def_id_p,
        def_type_p,
        entry_p,
        byref(handle),
    )
    return Request(handle)


def build_schema_request(
    submitter_did: str, schema: Union[bytes, str, dict]
) -> Request:
    """
    Builds a SCHEMA request to to add a credential schema to the ledger.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string
        schema: Credential schema:
            ```jsonc
            {
                "id": "<identifier of schema>",
                "attrNames": "<array of attribute name strings (the number of attributes
                    should be less or equal than 125)>",
                "name": "<schema's name string>",
                "version": "<schema's version string>",
                "ver": "<version of the schema json>"
            }```
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    schema_p = (
        encode_str(schema) if isinstance(schema, (str, bytes)) else encode_json(schema)
    )
    do_call("indy_vdr_build_schema_request", did_p, schema_p, byref(handle))
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


def build_rich_schema_request(
    submitter_did: str,
    rs_id: Union[bytes, str],
    rs_content: Union[bytes, str, dict],
    rs_name: Union[bytes, str],
    rs_version: Union[bytes, str],
    rs_type: Union[bytes, str],
    ver: Union[bytes, str],
) -> Request:
    """
    Builds a RICH_SCHEMA request to add it to the ledger.

    Args:
        submitter_did: Identifier (DID) of the transaction author as a base58-encoded
            string
        rs_id: identifier of the rich schema
        rs_content: JSON-LD string object
        rs_name: rich schema name
        rs_version: rich schema version
        rs_type: type constant as string, one of
            `ctx`, `sch`, `map`, `enc`, `cdf`, `pdf`
        ver: version as string
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    rs_id = encode_str(rs_id)
    rs_content = (
        encode_str(rs_content)
        if isinstance(rs_content, (str, bytes))
        else encode_json(rs_content)
    )
    rs_name = encode_str(rs_name)
    rs_version = encode_str(rs_version)
    rs_type = encode_str(rs_type)
    ver = encode_str(ver)
    do_call(
        "indy_vdr_build_rich_schema_request",
        did_p,
        rs_id,
        rs_content,
        rs_name,
        rs_version,
        rs_type,
        ver,
        byref(handle),
    )
    return Request(handle)


def build_get_rich_schema_object_by_id_request(
    submitter_did: str, rs_id: Union[bytes, str, dict]
) -> Request:
    """
    Builds a GET_RICH_SCHEMA_BY_ID request.

    Used to fetch a RICH_SCHEMA from the ledger using RICH_SCHEMA_ID.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string
        rs_id: DID-string like object which represents id of requested RICH_SCHEMA
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    rs_id = encode_str(rs_id) if isinstance(rs_id, (str, bytes)) else encode_json(rs_id)
    do_call(
        "indy_vdr_build_get_schema_object_by_id_request", did_p, rs_id, byref(handle)
    )
    do_call(
        "indy_vdr_build_get_rich_schema_object_by_id_request",
        did_p,
        rs_id,
        byref(handle),
    )
    return Request(handle)


def build_get_rich_schema_object_by_metadata_request(
    submitter_did: str,
    rs_type: Union[bytes, str],
    rs_name: Union[bytes, str],
    rs_version: Union[bytes, str],
) -> Request:
    """
    Builds a GET_RICH_SCHEMA_BY_METADATA request.

    Used to fetch a RICH_SCHEMA from the ledger using the RICH_SCHEMA's metadata.

    Args:
        submitter_did: Identifier (DID) of the transaction author as base58-encoded
            string
        rs_type: Rich Schema object's type enum
        rs_name: Rich Schema object's name,
        rs_version: Rich Schema object's version,
    """
    handle = RequestHandle()
    did_p = encode_str(submitter_did)
    rs_type = encode_str(rs_type)
    rs_name = encode_str(rs_name)
    rs_version = encode_str(rs_version)
    do_call(
        "indy_vdr_build_get_schema_object_by_metadata_request",
        did_p,
        rs_type,
        rs_name,
        rs_version,
        byref(handle),
    )
    do_call(
        "indy_vdr_build_get_rich_schema_object_by_metadata_request",
        did_p,
        rs_type,
        rs_name,
        rs_version,
        byref(handle),
    )
    return Request(handle)
