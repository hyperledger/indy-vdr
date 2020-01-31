from . import CustomRequest

test_req = {
    "identifier": "LibindyDid111111111111",
    "operation": {"data": 1, "ledgerId": 1, "type": "3"},
    "protocolVersion": 2,
    "reqId": 1579568148820684000,
}

req = CustomRequest(test_req)

print(req.body)
print(req.signature_input)
