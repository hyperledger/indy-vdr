import asyncio
from . import CustomRequest, Pool, LedgerType
from .bindings import get_version

print(get_version())

test_req = {
    "identifier": "LibindyDid111111111111",
    "operation": {"data": 1, "ledgerId": 1, "type": "3"},
    "protocolVersion": 2,
    "reqId": 1579568148820684000,
}

req = CustomRequest(test_req)
print(req.body)

sig_in = req.signature_input
print(sig_in)

# req.set_signature(bytes(32))
# print(req.body)

pool = Pool(genesis_path="genesis.txn")
print(f"created pool {pool}")


async def get_pool_txns():
    for txn in await pool.get_transactions():
        print(txn)


async def get_txn(seq_no: int):
    req = {"operation": {"data": seq_no, "ledgerId": LedgerType.DOMAIN, "type": "3"}}
    return await pool.submit_request(req)


async def get_txn_range(seq_nos):
    return [await get_txn(seq_no) for seq_no in seq_nos]


# asyncio.get_event_loop().run_until_complete(get_pool_txns())
print(asyncio.get_event_loop().run_until_complete(get_txn(11)))

print("done")
