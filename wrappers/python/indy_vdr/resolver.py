from . import bindings


class Resolver:
    """did:indy compliant resolver interface for a specific network."""

    def __init__(self, handle: bindings.ResolverHandle):
        """Initialize the pool instance."""
        self.handle = handle

    async def resolve(self, did: str) -> dict:
        """Resolve a DID to retrieve a DID Doc."""
        result = await bindings.resolve(self.handle, did)
        return result

    async def dereference(self, did_url: str) -> dict:
        """Dereference a DID Url to retrieve a ledger object."""
        result = await bindings.dereference(self.handle, did_url)
        return result
