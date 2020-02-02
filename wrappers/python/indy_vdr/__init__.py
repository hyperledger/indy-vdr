from abc import ABC, abstractmethod
from enum import IntEnum
from typing import List, Union

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
        pass

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


class Pool:
    def __init__(self, genesis_path: str, transactions=None):
        self.handle = bindings.pool_create_from_genesis_file(genesis_path)

    async def submit_request(
        self, request: Union[str, bytes, dict, BaseRequest]
    ) -> str:
        if not isinstance(request, BaseRequest):
            request = CustomRequest(request)
        if not self.handle:
            raise VdrError(None, "pool is closed")
        if not request.handle:
            raise VdrError(None, "no request handle")
        fut = bindings.pool_submit_request(self.handle, request.handle)
        request.handle = None  # request has been removed
        return await fut

    async def get_transactions(self) -> List[str]:
        if not self.handle:
            raise VdrError(None, "pool is closed")
        txns = await bindings.pool_get_transactions(self.handle)
        return txns.split("\n")

    def close(self):
        if self.handle:
            bindings.pool_close(self.handle)
            self.handle = None

    def __del__(self):
        self.close()
