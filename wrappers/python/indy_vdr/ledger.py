from abc import ABC, abstractmethod
from enum import IntEnum
from typing import Union

from . import bindings
from .error import VdrError


class LedgerType(IntEnum):
    POOL = 0
    DOMAIN = 1
    CONFIG = 2


class BaseRequest(ABC):
    def __init__(self):
        self.handle: bindings.RequestHandle = None

    @abstractmethod
    def build(self):
        raise NotImplementedError()

    @property
    def body(self):
        if not self.handle:
            raise VdrError(None, "no request handle")
        return bindings.request_get_body(self.handle)

    @property
    def signature_input(self):
        if not self.handle:
            raise VdrError(None, "no request handle")
        return bindings.request_get_signature_input(self.handle)

    def set_signature(self, signature: bytes):
        if not self.handle:
            raise VdrError(None, "no request handle")
        return bindings.request_set_signature(self.handle, signature)

    def __del__(self):
        if self.handle:
            bindings.request_free(self.handle)
            self.handle = None

    def __repr__(self):
        if not self.handle:
            return f"{self.__class__.__name__}(freed)"
        return super().__repr__(self)


class CustomRequest(BaseRequest):
    def __init__(self, body: Union[str, bytes, dict]):
        super().__init__()
        self.init_body = body
        self.build()

    def build(self):
        self.handle = bindings.build_custom_request(self.init_body)


class GetTxnRequest(BaseRequest):
    def __init__(self, ledger_type: int, seq_no: int, submitter_did: str = None):
        super().__init__()
        self.ledger_type = ledger_type
        self.seq_no = seq_no
        self.submitter_did = submitter_did
        self.build()

    def build(self):
        self.handle = bindings.build_get_txn_request(
            self.ledger_type, self.seq_no, self.submitter_did
        )


class GetValidatorInfoRequest(BaseRequest):
    def __init__(self, submitter_did: str):
        super().__init__()
        self.submitter_did = submitter_did
        self.build()

    def build(self):
        self.handle = bindings.build_get_validator_info_request(self.submitter_did)
