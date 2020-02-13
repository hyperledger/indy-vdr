import json
from typing import Mapping, Sequence, Union

from . import bindings
from .error import VdrError, VdrErrorCode
from .ledger import Request, build_custom_request


class Pool:
    def __init__(self, handle: bindings.PoolHandle):
        self.handle = handle
        self.last_status: dict = None

    def close(self):
        if hasattr(self, "handle") and self.handle:
            bindings.pool_close(self.handle)
            self.handle = None

    async def get_status(self) -> dict:
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        result = await bindings.pool_get_status(self.handle)
        self.last_status = json.loads(result)
        return result

    async def get_transactions(self) -> str:
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        return await bindings.pool_get_transactions(self.handle)

    async def refresh(self) -> dict:
        await bindings.pool_refresh(self.handle)
        result = await bindings.pool_get_status(self.handle)
        self.last_status = json.loads(result)
        return self.last_status

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
            mt_size = self.last_status and self.last_status.get("mt_size")
            status = f"{{handle: {self.handle.value}, mt_size: {mt_size}}}"
        else:
            status = "(closed)"
        return f"{self.__class__.__name__}{status}"


async def open_pool(
    transactions_path: str = None,
    transactions: str = None,
    node_weights: Mapping[str, float] = None,
    no_refresh: bool = False,
) -> Pool:
    if not (bool(transactions_path) ^ bool(transactions)):
        raise VdrError(
            VdrErrorCode.WRAPPER,
            "Must provide one of transactions or transactions_path",
        )
    params = {
        "transactions": transactions,
        "transactions_path": transactions_path,
        "node_weights": node_weights,
    }
    pool = Pool(bindings.pool_create(params))
    if not no_refresh:
        await pool.refresh()
    return pool
