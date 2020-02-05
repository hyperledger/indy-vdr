from enum import IntEnum


class VdrErrorCode(IntEnum):
    SUCCESS = 0
    FAILED = 1
    WRAPPER = 99


class VdrError(Exception):
    def __init__(self, code: VdrErrorCode, message: str, extra: str = None):
        super().__init__(message)
        self.code = code
        self.extra = extra
