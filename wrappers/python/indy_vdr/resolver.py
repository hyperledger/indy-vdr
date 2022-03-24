import json
import re
from typing import Dict

from . import bindings
from .error import VdrError, VdrErrorCode
from .pool import Pool, open_pool
from .utils import get_genesis_txns_from_did_indy_repo_by_name

DID_INDY_PREFIX = "did:indy"
NAMESPACE_PATTERN = "((?:[a-z0-9_-]+:?){1,2})"

DID_PATTERN = re.compile(f"^{DID_INDY_PREFIX}:{NAMESPACE_PATTERN}:(.*)")


class Resolver:
    """did:indy compliant resolver interface.

    Args:
        pool_map: Dict mapping ledger namespaces to pool instances
        autopilot: (default = False) If enabled resolver will try to
            fetch genesis txn from indy networks github when no pool instance
            is configured for the DID namespace
    """

    def __init__(self, pool_map: Dict[str, Pool] = {}, autopilot=False):
        """Initialize the resolver instance from a pool map."""
        self.pool_map = pool_map
        self.autopilot = autopilot

    def add_ledger(self, namespace: str, pool: Pool):
        """Add a ledger to the resolver."""
        self.pool_map[namespace] = pool

    def remove_ledger(self, namespace: str):
        """Remove a ledger from the resolver."""
        try:
            del self.pool_map[namespace]
        except KeyError:
            raise VdrError(VdrErrorCode.WRAPPER, f"Ledger {namespace} not configured")

    async def resolve(self, did: str) -> Dict:
        """Resolve a DID to retrieve a DID Doc."""
        namespace = get_namespace(did)
        pool = self.pool_map.get(namespace)
        if not pool:
            if self.autopilot:
                try:
                    pool_map = get_genesis_txns_from_did_indy_repo_by_name([namespace])
                    pool = await open_pool(pool_map[namespace])
                    self.add_ledger(namespace, pool)
                except KeyError:
                    raise VdrError(
                        VdrErrorCode.WRAPPER, f"Unknown DID namespace: {namespace}"
                    )

            else:
                raise VdrError(
                    VdrErrorCode.WRAPPER, f"Unknown DID namespace: {namespace}"
                )
        pool_handle = getattr(pool, "handle")
        result = await bindings.resolve(pool_handle, did)
        return json.loads(result)

    async def dereference(self, did_url: str) -> Dict:
        """Dereference a DID Url to retrieve a ledger object."""
        namespace = get_namespace(did_url)
        pool = self.pool_map.get(namespace)
        if not pool:
            if self.autopilot:
                try:
                    pool_map = get_genesis_txns_from_did_indy_repo_by_name([namespace])
                    pool = await open_pool(pool_map[namespace])
                    self.add_ledger(namespace, pool)
                except KeyError:
                    raise VdrError(
                        VdrErrorCode.WRAPPER, f"Unknown DID namespace: {namespace}"
                    )

            else:
                raise VdrError(
                    VdrErrorCode.WRAPPER, f"Unknown DID namespace: {namespace}"
                )
        pool_handle = getattr(pool, "handle")
        result = await bindings.dereference(pool_handle, did_url)
        return json.loads(result)


def get_namespace(did: str) -> str:
    matched = DID_PATTERN.match(did)
    if not matched:
        raise VdrError(VdrErrorCode.WRAPPER, f"Invalid DID: {did}")
    return matched.group(1)
