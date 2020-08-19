package vdr

import (
	"encoding/json"

	"github.com/mitchellh/mapstructure"
	"github.com/pkg/errors"
)

type ErrorReply struct {
	Op     string `json:"op"`
	Reason string `json:"reason"`
}

func (r ErrorReply) Error() string {
	return r.Reason
}

type ReadSuccessReply struct {
	Op     string     `json:"op"`
	Result *ReadReply `json:"result"`
}

type ReadReply struct {
	Type          string      `json:"type"`
	Identifier    string      `json:"identifier,omitempty"`
	ReqID         uint32      `json:"reqId"`
	SeqNo         uint32      `json:"seqNo"`
	TxnTime       uint32      `json:"TxnTime"`
	StateProof    StateProof  `json:"state_proof"`
	Data          interface{} `json:"data"`
	SignatureType string      `json:"signature_type,omitempty"`
	Origin        string      `json:"origin,omitempty"`
	Dest          string      `json:"dest,omitempty"`
	Ref           uint32      `json:"ref,omitempty"`
	Tag           string      `json:"tag,omitempty"`
}

type StateProof struct {
	RootHash        string     `json:"root_hash"`
	ProofNodes      string     `json:"proof_nodes"`
	MultiSignatures Signatures `json:"multi_signatures"`
}

type Signatures struct {
	Value        SigValue `json:"value"`
	Signature    string   `json:"signature"`
	Participants []string `json:"participants"`
}

type SigValue struct {
	Timestamp         int    `json:"timestamp"`
	LedgerID          int    `json:"ledger_id"`
	TxnRootHash       string `json:"txn_root_hash"`
	PoolStateRootHash string `json:"pool_state_root_hash"`
	StateRootHash     string `json:"state_root_hash"`
}

type WriteSuccessReply struct {
	Op     string      `json:"op"`
	Result *WriteReply `json:"result"`
}

type WriteReply struct {
	Ver          string                      `json:"ver"`
	Txn          WriteReplyResultTxn         `json:"txn"`
	TxnMetadata  WriteReplyResultTxnMetadata `json:"txnMetadata"`
	ReqSignature ReqSignature                `json:"reqSignature"`
	RootHash     string                      `json:"rootHash"`
	AuditPath    []string                    `json:"auditPath"`
}

type WriteReplyResultTxn struct {
	Type            string                 `json:"type"`
	ProtocolVersion int                    `json:"protocolVersion"`
	Data            map[string]interface{} `json:"data"`
	Metadata        map[string]interface{} `json:"metadata"`
}

type WriteReplyResultTxnMetadata struct {
	TxnTime int    `json:"txnTime"`
	SeqNo   int    `json:"seqNo"`
	TxnID   string `json:"txnId"`
}

type Metadata struct {
	TxnTime int    `json:"txnTime"`
	SeqNo   int    `json:"seqNo"`
	TxnID   string `json:"txnId"`
}
type ReqSignature struct {
	Type   string        `json:"type"`
	Values []ReqSigValue `json:"values"`
}

type ReqSigValue struct {
	From  string `json:"from"`
	Value string `json:"value"`
}

func parseReadReply(response string) (*ReadReply, error) {
	m := map[string]interface{}{}

	err := json.Unmarshal([]byte(response), &m)
	if err != nil {
		return nil, errors.Wrap(err, "unexpected error reading reply")
	}

	rply := m["op"].(string)
	switch rply {
	case "REQACK":
		return parseSuccessResponse(m)
	case "REQNACK":
		return nil, parseErrorResponse(m)
	case "REPLY":
		return parseSuccessResponse(m)
	case "REJECT":
		return nil, parseErrorResponse(m)
	default:
		return nil, errors.Errorf("unknown message reply: %s", rply)
	}

}
func parseWriteReply(response string) (*WriteReply, error) {
	m := map[string]interface{}{}

	err := json.Unmarshal([]byte(response), &m)
	if err != nil {
		return nil, errors.Wrap(err, "unexpected error reading reply")
	}

	rply := m["op"].(string)
	switch rply {
	case "REQACK":
		return parseSuccessWriteResponse(m)
	case "REQNACK":
		return nil, parseErrorResponse(m)
	case "REPLY":
		return parseSuccessWriteResponse(m)
	case "REJECT":
		return nil, parseErrorResponse(m)
	default:
		return nil, errors.Errorf("unknown message reply: %s", rply)
	}

}

func parseErrorResponse(m map[string]interface{}) error {
	out := ErrorReply{}
	dec, err := mapstructure.NewDecoder(&mapstructure.DecoderConfig{
		Result:  &out,
		TagName: "json",
	})
	if err != nil {
		return errors.Wrap(err, "unexpected error creating decoder for error parsing")
	}

	err = dec.Decode(m)
	if err != nil {
		return errors.Wrap(err, "unable to decode error message")
	}

	return out
}

func parseSuccessResponse(m map[string]interface{}) (*ReadReply, error) {
	out := ReadSuccessReply{}
	dec, err := mapstructure.NewDecoder(&mapstructure.DecoderConfig{
		Result:  &out,
		TagName: "json",
	})
	if err != nil {
		return nil, errors.Wrap(err, "unexpected error creating decoder for error parsing")
	}

	//TODO: consider inspecting m["Result"]["Data"] and perhaps request type and marshalling into
	//      something more specific that the caller can cast to...
	err = dec.Decode(m)
	if err != nil {
		return nil, errors.Wrap(err, "unable to decode success message")
	}

	return out.Result, nil
}

func parseSuccessWriteResponse(m map[string]interface{}) (*WriteReply, error) {
	out := WriteSuccessReply{}
	dec, err := mapstructure.NewDecoder(&mapstructure.DecoderConfig{
		Result:  &out,
		TagName: "json",
	})
	if err != nil {
		return nil, errors.Wrap(err, "unexpected error creating decoder for error parsing")
	}

	//TODO: consider inspecting m["Result"]["Data"] and perhaps request type and marshalling into
	//      something more specific that the caller can cast to...
	err = dec.Decode(m)
	if err != nil {
		return nil, errors.Wrap(err, "unable to decode success message")
	}

	return out.Result, nil
}
