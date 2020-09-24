"""Handling of ledger pool instances."""

import json
from datetime import datetime
from typing import Mapping, Sequence, Union

from . import bindings
from .error import VdrError, VdrErrorCode
from .ledger import Request, build_custom_request


class Pool:
    """An opened ledger pool instance."""

    def __init__(self, handle: bindings.PoolHandle):
        """Initialize the pool instance."""
        self.handle = handle
        self.last_refresh: datetime = None
        self.last_status: dict = None

    def close(self):
        """Close and free the pool instance."""
        if hasattr(self, "handle") and self.handle:
            bindings.pool_close(self.handle)
            self.handle = None

    async def get_status(self) -> dict:
        """Get the current status of the pool instance."""
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        result = await bindings.pool_get_status(self.handle)
        self.last_status = json.loads(result)
        return result

    async def get_transactions(self) -> str:
        """Get the current pool transactions of the pool instance."""
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        return await bindings.pool_get_transactions(self.handle)

    async def get_verifiers(self) -> dict:
        """Get the current set of active verifiers for the pool instance."""
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        return json.loads(await bindings.pool_get_verifiers(self.handle))

    @property
    def last_refresh_seconds(self) -> float:
        """Get the number of seconds since the last verifier pool refresh."""
        return (
            (datetime.now() - self.last_refresh).total_seconds()
            if self.last_refresh
            else None
        )

    async def refresh(self) -> dict:
        """Check the verifier pool and load any new pool transactions."""
        await bindings.pool_refresh(self.handle)
        result = await bindings.pool_get_status(self.handle)
        self.last_status = json.loads(result)
        self.last_refresh = datetime.now()
        return self.last_status

    async def submit_action(
        self,
        request: Union[str, bytes, dict, Request],
        node_aliases: Sequence[str] = None,
        timeout: int = None,
    ) -> dict:
        """Submit a pool action to all verifier nodes.

        The following requests are sent as actions:
            GET_VALIDATOR_INFO
            POOL_RESTART

        Args:
            request: May be a prepared `Request` instance, a JSON string or bytes
                instance, or a dict representing a new custom ledger request

        Returns:
            A dict with the node aliases as keys and the node's responses
            as values
        """
        if not isinstance(request, Request):
            request = build_custom_request(request)
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "pool is closed")
        if not request.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        fut = bindings.pool_submit_action(
            self.handle, request.handle, node_aliases, timeout
        )
        request.handle = None  # request has been removed
        result = await fut
        return json.loads(result)

    async def submit_request(self, request: Union[str, bytes, dict, Request]) -> dict:
        """Submit a ledger request.

        Args:
            request: May be a prepared `Request` instance, a JSON string or bytes
                instance, or a dict representing a new custom ledger request

        Returns:
            A dict representing the decoded JSON response
        """
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
        """Close the pool instance when there are no more references to this object."""
        self.close()

    def __repr__(self) -> str:
        """Format the pool instance as a debug string."""
        if self.handle:
            last_refresh = self.last_refresh_seconds
            last_refresh = int(last_refresh) if last_refresh is not None else None
            mt_size = self.last_status and self.last_status.get("mt_size")
            node_count = len(self.last_status and self.last_status.get("nodes") or ())
            status = (
                f"({self.handle.value}, count={node_count}, "
                f"last_refresh={last_refresh}, mt_size={mt_size})"
            )
        else:
            status = "(closed)"
        return f"{self.__class__.__name__}{status}"


async def open_pool(
    transactions_path: str = None,
    transactions: str = None,
    node_weights: Mapping[str, float] = None,
    no_refresh: bool = False,
) -> Pool:
    """Create a new ledger pool instance.

    Either `transactions` or `transactions_path` must be specified, but not both.

    Args:
        transactions_path: The path to a genesis transactions file
        transactions: A JSON string representing the genesis transactions
        node_weights: A dict with node aliases as the keys and priority weights
            as the values. The default weight is 1.0, so higher weights give the
            node a higher probability of being selected. A weight of zero means the
            node will never be selected.
        no_refresh: Disable the initial verifier pool refresh

    Returns:
        A new `Pool` instance which may be used to submit ledger requests
    """
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
