import asyncio
import json
import sys
import os
import urllib.request

from indy_vdr.bindings import version
from indy_vdr.ledger import (
    build_custom_request,
    build_get_txn_request,
    build_get_acceptance_mechanisms_request,
    build_get_txn_author_agreement_request,
    build_get_validator_info_request,
    build_get_cred_def_request,
    build_get_revoc_reg_def_request,
    build_get_revoc_reg_request,
    build_get_revoc_reg_delta_request,
    build_get_schema_request,
    build_node_request,
    build_pool_config_request,
    build_pool_restart_request,
    build_auth_rule_request,
    build_auth_rules_request,
    build_get_auth_rule_request,
    build_ledgers_freeze_request,
    build_get_frozen_ledgers_request,
    # build_revoc_reg_entry_request,
    # build_rich_schema_request,
    # build_get_schema_object_by_id_request,
    # build_get_schema_object_by_metadata_request,
    prepare_txn_author_agreement_acceptance,
    LedgerType,
)
from indy_vdr.pool import Pool, open_pool


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
    log("Created pool:", pool)

    verifiers = await pool.get_verifiers()
    log("Verifiers:", verifiers)

    #    req =  build_revoc_reg_entry_request(
    #     "CkTEG6qiypB8TCDS5mxRmy", "CkTEG6qiypB8TCDS5mxRmy:4:CkTEG6qiypB8TCDS5mxRmy:3:CL:67559:default:CL_ACCUM:e3abc098-749f-4c4a-a5f7-4e518035e820", "CL_ACCUM", '{"ver": "1.0", "value": {"accum": "21 117FA38C35FB5D721113285DC65741A227E860EA97195706A8EEEE778DE2A1013 21 137E20CCC0D5E63B79B1A392EBCC93A855EE6F80D95121A1F600F1FE11E5CB005 6 6D00C839527AE3B7E26B32A1AEACCA03A5415FF04A9ADA0D164E64E95AF7DAD0 4 02E046D5BBFC929582E79655B62FEEA65C86DC7EE1D6A7BE0A56E8BBF52FCCEB 6 70809B1DE08116FFDD5F91168EC5B87B2BD12E47A8952B2112A643500AB88D57 4 25AE5E9FEDC0BAC16FE765249B647ED69A5E73C5F956C34ADD0DAC5A4E4B2A95"}}')
    #     print(req)
    #     return

    test_req = {
        "operation": {"data": 1, "ledgerId": 1, "type": "3"},
        "protocolVersion": 2,
        "reqId": 123,
        "identifier": "LibindyDid111111111111",
    }
    req = build_custom_request(test_req)
    log("Custom request body:", req.body)
    #
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
    req.set_multi_signature("V4SGRU86Z58d6TV7PBUe6f", b"sig")
    log("Request with TAA acceptance and endorser:", req.body)

    # req = build_disable_all_txn_author_agreements_request("V4SGRU86Z58d6TV7PBUe6f")
    # log(await pool.submit_request(req))

    txn = await get_txn(pool, 1)
    log(json.dumps(txn, indent=2))

    req = build_get_schema_request(
        None, "6qnvgJtqwK44D8LFYnV5Yf:2:relationship.dflow:1.0.0"
    )
    log("Get schema request:", req.body)

    req = build_get_cred_def_request(None, "A9Rsuu7FNquw8Ne2Smu5Nr:3:CL:15:tag")
    log("Get cred def request:", req.body)

    revoc_id = (
        "L5wx9FUxCDpFJEdFc23jcn:4:L5wx9FUxCDpFJEdFc23jcn:3:CL:1954:"
        "default:CL_ACCUM:c024e30d-f3eb-42c5-908a-ff885667588d"
    )

    req = build_get_revoc_reg_def_request(None, revoc_id)
    log("Get revoc reg def request:", req.body)

    req = build_get_revoc_reg_request(None, revoc_id, timestamp=1)
    log("Get revoc reg request:", req.body)

    req = build_get_revoc_reg_delta_request(None, revoc_id, from_ts=None, to_ts=1)
    log("Get revoc reg delta request:", req.body)

    identifier = "V4SGRU86Z58d6TV7PBUe6f"
    dest = "V4SGRU86Z58d6TV7PBUe6f"
    data = {
        "node_ip": "ip",
        "node_port": 1,
        "client_ip": "ip",
        "client_port": 1,
        "alias": "some",
        "services": ["VALIDATOR"],
        "blskey": "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW",
    }
    req = build_node_request(identifier, dest, data)
    log("Node request:", req.body)

    req = build_pool_config_request(identifier, True, False)
    log("Pool Config request:", req.body)

    req = build_pool_restart_request(identifier, "start", None)
    log("Pool Restart request:", req.body)

    txn_type = "NYM"
    auth_type = "1"
    auth_action = "ADD"
    field = "role"
    old_value = "0"
    new_value = "101"
    constraint = {
        "sig_count": 1,
        "metadata": {},
        "role": "0",
        "constraint_id": "ROLE",
        "need_to_be_owner": False,
    }
    req = build_auth_rule_request(
        identifier, txn_type, auth_action, field, old_value, new_value, constraint
    )
    log("Auth Rule request:", req.body)

    rules = [
        {
            "auth_type": auth_type,
            "auth_action": auth_action,
            "field": field,
            "new_value": new_value,
            "constraint": constraint,
        },
    ]
    req = build_auth_rules_request(identifier, rules)
    log("Auth Rules request:", req.body)

    req = build_get_auth_rule_request(
        identifier, auth_type, auth_action, field, None, new_value
    )
    log("Get Auth Rule request:", req.body)

    ledgers_ids = [1, 10, 100]
    req = build_ledgers_freeze_request(identifier, ledgers_ids)
    log("Ledgers Freeze request:", req.body)

    req = build_get_frozen_ledgers_request(identifier)
    log("Get Frozen Ledgers request:", req.body)

    # req = build_rich_schema_request(
    #     None, "did:sov:some_hash", '{"some": 1}', "test", "version", "sch", "1.0.0"
    # )
    # log("Get rich schema request:", req.body)

    # req = build_get_schema_object_by_id_request(None, "did:sov:some_hash")
    # log("Get rich schema GET request by ID:", req.body)
    #
    # req = build_get_schema_object_by_metadata_request(None, "sch", "test", "1.0.0")
    # log("Get rich schema GET request by Metadata:", req.body)


def get_script_dir():
    return os.path.dirname(os.path.realpath(__file__))


def download_buildernet_genesis_file():
    genesis_file_url = (
        "https://raw.githubusercontent.com/sovrin-foundation/"
        "sovrin/master/sovrin/pool_transactions_builder_genesis"
    )
    target_local_path = f"{get_script_dir()}/genesis_sov_buildernet.txn"
    urllib.request.urlretrieve(genesis_file_url, target_local_path)
    return target_local_path


if __name__ == "__main__":
    log("indy-vdr version:", version())

    genesis_path = (
        sys.argv[1] if len(sys.argv) > 1 else download_buildernet_genesis_file()
    )
    asyncio.get_event_loop().run_until_complete(basic_test(genesis_path))
