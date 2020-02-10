import json
from typing import Sequence, List, Union

from . import bindings
from .error import VdrError, VdrErrorCode
from .ledger import Request, build_custom_request


class Pool:
    def __init__(self, genesis_path: str, transactions=None):
        self.handle = None
        self.handle = bindings.pool_create_from_genesis_file(genesis_path)

    def close(self):
        if self.handle:
            bindings.pool_close(self.handle)
            self.handle = None

    async def get_status(self) -> dict:
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        result = await bindings.pool_get_status(self.handle)
        return json.loads(result)

    async def get_transactions(self) -> List[str]:
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        txns = await bindings.pool_get_transactions(self.handle)
        return txns.split("\n")

    async def refresh(self) -> dict:
        await bindings.pool_refresh(self.handle)
        return await bindings.pool_get_status(self.handle)

    async def submit_action(
        self,
        request: Union[str, bytes, dict, Request],
        nodes: Sequence[str] = None,
        timeout: int = None,
    ) -> str:
        if not isinstance(request, Request):
            request = build_custom_request(request)
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        if not request.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        fut = bindings.pool_submit_action(self.handle, request.handle, nodes, timeout)
        request.handle = None  # request has been removed
        result = await fut
        return json.loads(result)

    async def submit_request(self, request: Union[str, bytes, dict, Request]) -> dict:
        if not isinstance(request, Request):
            request = build_custom_request(request)
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        if not request.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        fut = bindings.pool_submit_request(self.handle, request.handle)
        request.handle = None  # request has been removed
        result = await fut
        # FIXME improve handling of bad request
        return json.loads(result)["result"]

    def __del__(self):
        self.close()

    def __repr__(self):
        if self.handle:
            status = self.handle
        else:
            status = "closed"
        return f"{self.__class__.__name__}({status})"
