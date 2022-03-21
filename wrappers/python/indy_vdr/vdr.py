from . import bindings
from .error import VdrError, VdrErrorCode

class Vdr:
    """An opened vdr instance."""

    def __init__(self, handle: bindings.VdrHandle):
        """Initialize the vdr instance."""
        self.handle = handle
    
    async def resolve(self, did: str) -> dict:
        """Resolve a DID to retrieve a DID Doc."""
        result = await bindings.vdr_resolve(self.handle, did)
        return result

    # FIXME: Not yet implemented
    # async def refresh_all(self) -> dict:
    #     """Refresh all validator pools."""
    #     result = await bindings.vdr_refresh(self.handle)
    #     return result


def init_from_github() -> Vdr:
    """Initialize indy vdr with standard networks."""
    vdr = Vdr(bindings.vdr_create_from_github())
    return vdr

def init_from_folder(path: str, genesis_filename = None) -> Vdr:
    """Initialize indy vdr from local folder."""
    vdr = Vdr(bindings.vdr_create_from_folder(path, genesis_filename))
    return vdr

