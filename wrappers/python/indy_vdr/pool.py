import json
from typing import Sequence, List, Union

from . import bindings
from .error import VdrError
from .ledger import BaseRequest, CustomRequest


class Pool:
    def __init__(self, genesis_path: str, transactions=None):
        self.handle = None
        self.handle = bindings.pool_create_from_genesis_file(genesis_path)

    async def submit_action(
        self,
        request: Union[str, bytes, dict, BaseRequest],
        nodes: Sequence[str] = None,
        timeout: int = None,
    ) -> str:
        if not isinstance(request, BaseRequest):
            request = CustomRequest(request)
        if not self.handle:
            raise VdrError(None, "pool is closed")
        if not request.handle:
            raise VdrError(None, "no request handle")
        fut = bindings.pool_submit_action(self.handle, request.handle, nodes, timeout)
        request.handle = None  # request has been removed
        result = await fut
        return json.loads(result)

    async def submit_request(
        self, request: Union[str, bytes, dict, BaseRequest]
    ) -> dict:
        if not isinstance(request, BaseRequest):
            request = CustomRequest(request)
        if not self.handle:
            raise VdrError(None, "pool is closed")
        if not request.handle:
            raise VdrError(None, "no request handle")
        fut = bindings.pool_submit_request(self.handle, request.handle)
        request.handle = None  # request has been removed
        result = await fut
        # FIXME improve handling of bad request
        return json.loads(result)["result"]

    async def get_transactions(self) -> List[str]:
        if not self.handle:
            raise VdrError(None, "pool is closed")
        txns = await bindings.pool_get_transactions(self.handle)
        return txns.split("\n")

    async def refresh(self):
        return await bindings.pool_refresh(self.handle)

    def close(self):
        if self.handle:
            bindings.pool_close(self.handle)
            self.handle = None

    def __del__(self):
        self.close()