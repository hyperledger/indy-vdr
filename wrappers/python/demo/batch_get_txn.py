import asyncio
import json
import logging
import os
import sys
import time

from asyncio import Event, Task
from typing import List, NamedTuple
import urllib.request

from indy_vdr.ledger import (
    build_get_txn_request,
    LedgerType,
)
from indy_vdr.pool import Pool, open_pool

MAX_PENDING = 25
MAX_RETRY = 10

logging.basicConfig(level=logging.INFO, stream=sys.stderr)
LOGGER = logging.getLogger(__name__)


class GetTxn(NamedTuple):
    seq_no: int
    task: Task


async def get_txn(pool: Pool, seq_no: int):
    retries = MAX_RETRY
    while True:
        req = build_get_txn_request(None, LedgerType.DOMAIN, seq_no)
        try:
            return await pool.submit_request(req)
        except Exception:
            retry = retries > 0
            LOGGER.exception(
                f"Error fetching transaction (seqno: {seq_no}, retry: {retry})"
            )
            if not retry:
                raise
            retries -= 1


async def get_max_seq_no(pool):
    try:
        txn = await get_txn(pool, 1)
        return txn["data"]["ledgerSize"]
    except Exception as e:
        raise Exception("Error fetching maximum sequence number") from e


async def get_txn_range(pool: Pool, start_txn: int, end_txn: int = None):
    if not end_txn:
        end_txn = await get_max_seq_no(pool)

    loop = asyncio.get_event_loop()
    seq_no = start_txn
    requests: List[GetTxn] = []
    updated = Event()

    def on_update(_task):
        updated.set()

    while True:
        updated.clear()

        # check pending requests
        while requests and requests[0].task.done():
            task = requests.pop(0).task
            # may raise an exception if failed after retries
            yield task.result()

        while len(requests) < MAX_PENDING and (end_txn is None or seq_no < end_txn):
            task = loop.create_task(get_txn(pool, seq_no))
            task.add_done_callback(on_update)
            requests.append(GetTxn(seq_no, task))
            seq_no += 1

        if not requests:
            break

        await updated.wait()


async def print_txn_range(transactions_path, start_txn: int, end_txn: int = None):
    LOGGER.info("Opening pool")
    pool = await open_pool(transactions_path=transactions_path)

    LOGGER.info("Starting retrieval")
    start = time.perf_counter()
    count = 0

    async for result in get_txn_range(pool, start_txn, end_txn):
        print(json.dumps(result))
        count += 1

    dur = time.perf_counter() - start
    LOGGER.info(f"Retrieved {count} transactions in {dur:0.2f}s")


def get_script_dir():
    return os.path.dirname(os.path.realpath(__file__))


def download_buildernet_genesis_file():
    genesis_file_url = (
        "https://raw.githubusercontent.com/sovrin-foundation/"
        "sovrin/master/sovrin/pool_transactions_builder_genesis"
    )
    target_local_path = f"{get_script_dir()}/genesis_sov_buildernet.txn"
    LOGGER.info("Fetching genesis transactions")
    urllib.request.urlretrieve(genesis_file_url, target_local_path)
    return target_local_path


if __name__ == "__main__":
    genesis_path = (
        sys.argv[1] if len(sys.argv) > 1 else download_buildernet_genesis_file()
    )
    asyncio.get_event_loop().run_until_complete(print_txn_range(genesis_path, 1, 100))
