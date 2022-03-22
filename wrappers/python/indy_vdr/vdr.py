from . import bindings

from typing import List

class Vdr:
    """An opened vdr instance."""

    def __init__(self, handle: bindings.VdrHandle):
        """Initialize the vdr instance."""
        self.handle = handle

    def get_ledgers(self) -> List[str]:
        result = bindings.vdr_get_ledgers(self.handle)
        return result
    
    async def resolve(self, did: str) -> dict:
        """Resolve a DID to retrieve a DID Doc."""
        result = await bindings.vdr_resolve(self.handle, did)
        return result


    async def refresh(self, ledger: str):
        """Refresh a validator pool."""
        await bindings.vdr_refresh(self.handle, ledger)


def init_from_github() -> Vdr:
    """Initialize indy vdr with standard networks.
    This is only available if indy vdr is compiled with
    feature git.
    """
    vdr = Vdr(bindings.vdr_create_from_github())
    return vdr

def init_from_folder(path: str, genesis_filename = None) -> Vdr:
    """Initialize indy vdr from local folder."""
    vdr = Vdr(bindings.vdr_create_from_folder(path, genesis_filename))
    return vdr

