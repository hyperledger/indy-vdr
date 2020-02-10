from enum import IntEnum
from typing import Optional, Union

from . import bindings
from .error import VdrError, VdrErrorCode


class LedgerType(IntEnum):
    POOL = 0
    DOMAIN = 1
    CONFIG = 2


class Request:
    def __init__(self, handle: bindings.RequestHandle):
        self.handle = handle

    @property
    def body(self):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        return bindings.request_get_body(self.handle)

    @property
    def signature_input(self):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        return bindings.request_get_signature_input(self.handle)

    def set_signature(self, signature: bytes):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        return bindings.request_set_signature(self.handle, signature)

    def __del__(self):
        if self.handle:
            bindings.request_free(self.handle)
            self.handle = None

    def __repr__(self):
        if not self.handle:
            return f"{self.__class__.__name__}(freed)"
        return super().__repr__(self)


async def build_acceptance_mechanisms_request(
    submitter_did: str,
    aml: Union[str, bytes, dict],
    version: str,
    aml_context: str = None,
) -> Request:
    return Request(
        bindings.build_acceptance_mechanisms_request(
            submitter_did, aml, version, aml_context
        )
    )


def build_custom_request(body: Union[str, bytes, dict]) -> Request:
    return Request(bindings.build_custom_request(body))


def build_disable_all_txn_author_agreements_request(submitter_did: str) -> Request:
    return Request(
        bindings.build_disable_all_txn_author_agreements_request(submitter_did)
    )


def build_get_acceptance_mechanisms_request(
    timestamp: int = None, version: str = None, submitter_did: str = None
) -> Request:
    return Request(
        bindings.build_get_acceptance_mechanisms_request(
            submitter_did, timestamp, version
        )
    )


def build_get_nym_request(nym: str, submitter_did: str = None) -> Request:
    return Request(bindings.build_get_nym_request(submitter_did, nym))


def build_get_txn_author_agreement_request(
    data: Union[bytes, str, dict] = None, submitter_did: str = None
) -> Request:
    return Request(bindings.build_get_txn_author_agreement_request(submitter_did, data))


def build_get_txn_request(
    ledger_type: int, seq_no: int, submitter_did: str = None
) -> Request:
    return Request(bindings.build_get_txn_request(submitter_did, ledger_type, seq_no))


def build_get_validator_info_request(submitter_did: str) -> Request:
    return Request(bindings.build_get_validator_info_request(submitter_did))


def build_nym_request(
    submitter_did: str,
    nym: str,
    verkey: str = None,
    alias: str = None,
    role: str = None,
) -> Request:
    return Request(bindings.build_nym_request(submitter_did, nym, verkey, alias, role))


def build_txn_author_agreement_request(
    submitter_did: str,
    text: Optional[str],
    version: str,
    ratification_ts: int = None,
    retirement_ts: int = None,
) -> Request:
    return Request(
        bindings.build_txn_author_agreement_request(
            submitter_did, text, version, ratification_ts, retirement_ts
        )
    )
