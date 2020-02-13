from typing import Union

from . import bindings
from .error import VdrError, VdrErrorCode


class Request:
    def __init__(self, handle: bindings.RequestHandle):
        self.handle = handle

    @property
    def body(self):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        return bindings.request_get_body(self.handle)

    def free(self):
        if hasattr(self, "handle") and self.handle:
            bindings.request_free(self.handle)
            self.handle = None

    @property
    def signature_input(self):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        return bindings.request_get_signature_input(self.handle)

    def set_endorser(self, endorser: str):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        bindings.request_set_endorser(self.handle, endorser)

    def set_signature(self, signature: bytes):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        bindings.request_set_signature(self.handle, signature)

    def set_taa_acceptance(self, acceptance: Union[str, dict]):
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        bindings.request_set_taa_acceptance(self.handle, acceptance)

    def __del__(self):
        self.free()

    def __repr__(self):
        if self.handle:
            status = self.handle
        else:
            status = "freed"
        return f"{self.__class__.__name__}({status})"
