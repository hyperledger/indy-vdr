import asyncio
import json
from .bindings import get_version
from .ledger import CustomRequest, LedgerType
from .pool import Pool


def log(*args):
    print(*args, "\n")


async def get_pool_txns():
    for txn in await pool.get_transactions():
        print(txn)


async def get_txn(seq_no: int):
    req = {"operation": {"data": seq_no, "ledgerId": LedgerType.DOMAIN, "type": "3"}}
    result = await pool.submit_request(req)
    body = json.loads(result)["result"]
    return body


async def get_txn_range(seq_nos):
    return [await get_txn(seq_no) for seq_no in seq_nos]


log("indy-vdr version:", get_version())

test_req = {"operation": {"data": 1, "ledgerId": 1, "type": "3"}}

req = CustomRequest(test_req)
log("Request body:", req.body)

sig_in = req.signature_input
log("Signature input:", sig_in)

pool = Pool(genesis_path="../../bctest.txn")
log(f"Created pool {pool}")

txn = asyncio.get_event_loop().run_until_complete(get_txn(11))
log(json.dumps(txn, indent=2))
