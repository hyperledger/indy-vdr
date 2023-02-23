import json
import re
from typing import Dict

from . import bindings
from .error import VdrError, VdrErrorCode
from .ledger import build_get_attrib_request
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
        result = json.loads(await bindings.resolve(pool_handle, did))

        reply_data = json.loads(
            result["didDocumentMetadata"]["nodeResponse"]["result"]["data"]
        )
        diddoc_content = reply_data.get("diddocContent", None)

        # Handle legacy case, where diddocContent is not present and we want to check for
        # associated ATTRIB endpoints. We can't handle in this in libindy_vdr directly.
        if not diddoc_content:
            unqualified_did = reply_data["dest"]
            # Find out if specific version was requested
            seq_no = result["didDocumentMetadata"]["nodeResponse"]["result"].get(
                "seqNo", None
            )
            timestamp = result["didDocumentMetadata"]["nodeResponse"]["result"].get(
                "timestamp", None
            )
            if timestamp:
                seq_no = None
            req = build_get_attrib_request(
                None, unqualified_did, "endpoint", None, None, seq_no, timestamp
            )
            res = await pool.submit_request(req)
            data = res.get("data", None)
            if data:
                data = json.loads(data)
                endpoints = data.get("endpoint", None)

                if endpoints:
                    services = []
                    for (service_type, service_endpoint) in endpoints.items():
                        if service_type == "endpoint":

                            services.append(
                                {
                                    "id": f"did:indy:{namespace}:{unqualified_did}#did-communication",
                                    "type": "did-communication",
                                    "recipientKeys": [
                                        f"did:indy:{namespace}:{unqualified_did}#verkey"
                                    ],
                                    "routingKeys": [],
                                    "priority": 0,
                                }
                            )

                        else:

                            services.append(
                                {
                                    "id": f"did:indy:{namespace}:{unqualified_did}#{service_type}",
                                    "type": service_type,
                                    "serviceEndpoint": service_endpoint,
                                }
                            )
                    result["didDocument"]["services"] = services

        return result

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
