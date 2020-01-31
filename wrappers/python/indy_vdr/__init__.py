from abc import ABC, abstractmethod
from typing import Union

from . import bindings


class BaseRequest(ABC):
    def __init__(self):
        self.handle: bindings.RequestHandle = None

    @abstractmethod
    def build(self):
        pass

    @property
    def body(self):
        if not self.handle:
            raise Exception("request not built")
        return bindings.request_get_body(self.handle)

    @property
    def signature_input(self):
        if not self.handle:
            raise Exception("request not built")
        return bindings.request_get_signature_input(self.handle)

    def set_signature(self, signature: bytes):
        if not self.handle:
            raise Exception("request not built")
        return bindings.request_set_signature(self.handle, signature)


class CustomRequest(BaseRequest):
    def __init__(self, body: Union[str, bytes, dict]):
        super().__init__()
        self.init_body = body
        self.build()

    def build(self):
        self.handle = bindings.build_custom_request(self.init_body)
