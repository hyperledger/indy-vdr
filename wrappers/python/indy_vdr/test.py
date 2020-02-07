import asyncio
import json
import sys

from .bindings import get_version
from .ledger import CustomRequest, GetTxnRequest, GetValidatorInfoRequest, LedgerType
from .pool import Pool


def log(*args):
    print(*args, "\n")


async def get_pool_txns():
    for txn in await pool.get_transactions():
        print(txn)


async def get_txn(pool: Pool, seq_no: int):
    req = GetTxnRequest(LedgerType.DOMAIN, seq_no)
    return await pool.submit_request(req)


async def get_txn_range(pool: Pool, seq_nos):
    return [await get_txn(pool, seq_no) for seq_no in seq_nos]


async def get_validator_info(pool: Pool):
    req = GetValidatorInfoRequest("V4SGRU86Z58d6TV7PBUe6f")
    return await pool.submit_action(req)


if __name__ == "__main__":
    log("indy-vdr version:", get_version())

    genesis_path = len(sys.argv) > 1 and sys.argv[1] or "genesis.txn"

    pool = Pool(genesis_path="genesis.txn")
    log(f"Created pool: {pool}")

    test_req = {"operation": {"data": 1, "ledgerId": 1, "type": "3"}}
    req = CustomRequest(test_req)
    log("Custom request body:", req.body)

    sig_in = req.signature_input
    log("Custom request signature input:", sig_in)

    print("Refreshing pool")
    status = asyncio.get_event_loop().run_until_complete(pool.refresh())
    log("Pool status:", status)

    txn = asyncio.get_event_loop().run_until_complete(get_txn(pool, 11))
    log(json.dumps(txn, indent=2))
