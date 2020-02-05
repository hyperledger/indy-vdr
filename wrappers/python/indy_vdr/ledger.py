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


class CustomRequest(BaseRequest):
    def __init__(self, body: Union[str, bytes, dict]):
        super().__init__()
        self.init_body = body
        self.build()

    def build(self):
        self.handle = bindings.build_custom_request(self.init_body)
