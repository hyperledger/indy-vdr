"""Error classes."""

from enum import IntEnum


class VdrErrorCode(IntEnum):
    SUCCESS = 0
    CONFIG = 1
    CONNECTION = 2
    FILESYSTEM = 3
    INPUT = 4
    RESOURCE = 5
    UNAVAILABLE = 6
    UNEXPECTED = 7
    POOL_NO_CONSENSUS = 30
    POOL_REQUEST_FAILED = 31
    POOL_TIMEOUT = 32
    WRAPPER = 99


class VdrError(Exception):
    def __init__(self, code: VdrErrorCode, message: str, extra: str = None):
        super().__init__(message)
        self.code = code
        self.extra = extra
