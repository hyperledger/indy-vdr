import asyncio
import json
import sys

from .bindings import version
from .ledger import (
    build_custom_request,
    build_get_txn_request,
    build_get_acceptance_mechanisms_request,
    build_get_txn_author_agreement_request,
    build_get_validator_info_request,
    prepare_txn_author_agreement_acceptance,
    LedgerType,
)
from .pool import Pool, open_pool


def log(*args):
    print(*args, "\n")


async def get_pool_txns(pool: Pool):
    for txn in await pool.get_transactions():
        print(txn)


async def get_txn(pool: Pool, seq_no: int):
    req = build_get_txn_request(None, LedgerType.DOMAIN, seq_no)
    return await pool.submit_request(req)


async def get_txn_range(pool: Pool, seq_nos):
    return [await get_txn(pool, seq_no) for seq_no in seq_nos]


async def get_validator_info(pool: Pool):
    req = build_get_validator_info_request("V4SGRU86Z58d6TV7PBUe6f")
    return await pool.submit_action(req)


async def basic_test(transactions_path):
    pool = await open_pool(transactions_path=transactions_path)
    log(f"Created pool: {pool}")

    test_req = {"operation": {"data": 1, "ledgerId": 1, "type": "3"}}
    req = build_custom_request(test_req)
    log("Custom request body:", req.body)

    sig_in = req.signature_input
    log("Custom request signature input:", sig_in)

    req = build_get_txn_author_agreement_request()
    log(await pool.submit_request(req))

    req = build_get_acceptance_mechanisms_request()
    log(await pool.submit_request(req))

    acceptance = prepare_txn_author_agreement_acceptance(
        "acceptance text", "1.1.1", None, mechanism="manual"
    )
    req = build_get_txn_request(None, 1, 15)
    req.set_txn_author_agreement_acceptance(acceptance)
    req.set_endorser("V4SGRU86Z58d6TV7PBUe6f")
    log("Request with TAA acceptance and endorser:", req.body)

    # req = build_disable_all_txn_author_agreements_request("V4SGRU86Z58d6TV7PBUe6f")
    # log(await pool.submit_request(req))

    txn = await get_txn(pool, 11)
    log(json.dumps(txn, indent=2))


if __name__ == "__main__":
    log("indy-vdr version:", version())

    genesis_path = len(sys.argv) > 1 and sys.argv[1] or "genesis.txn"

    asyncio.get_event_loop().run_until_complete(basic_test(genesis_path))
