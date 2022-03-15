/*
Copyright Scoir Inc. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
*/

package vdr

/*
#cgo LDFLAGS: -lindy_vdr
#include "libindy_vdr.h"
#include <stdlib.h>

extern void submitRequest(ErrorCode err, const char* response);
typedef void (*submitRequestWrapper)(ErrorCode err, const char* response);

extern void refresh(ErrorCode err);
typedef void (*refreshWrapper)(ErrorCode err);

extern void status(ErrorCode err, const char* response);
typedef void (*statusWrapper)(ErrorCode err, const char* response);

*/
import "C"

import (
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"sync"
	"unsafe"

	"github.com/mr-tron/base58"
	"github.com/pkg/errors"
)

type Handle C.int64_t

type Client struct {
	pool        Handle
	genesisTxns []byte
}

//New creates an Indy IndyVDR client connected to the Indy distributed ledger identified by the genesis file
//provided as a reader.
func New(genesis io.ReadCloser) (*Client, error) {

	txns, err := ioutil.ReadAll(genesis)
	if err != nil {
		return nil, fmt.Errorf("reading genesis file failed: %w", err)
	}

	params := map[string]interface{}{
		"transactions": string(txns),
	}

	d, err := json.Marshal(params)
	if err != nil {
		return nil, fmt.Errorf("formatting json params to indy failed: %w", err)
	}

	var pool C.int64_t
	cparams := C.CString(string(d))
	result := C.indy_vdr_pool_create(cparams, &pool)
	C.free(unsafe.Pointer(cparams))
	if result != 0 {
		return nil, fmt.Errorf("open indy pool failed. (Indy error code: [%v])", result)
	}

	out := &Client{
		pool:        Handle(pool),
		genesisTxns: txns,
	}
	return out, nil
}

//Genesis returns the genesis file of the network to which this client is connected
func (r *Client) Genesis() []byte {
	return r.genesisTxns
}

//Close shuts down the connection and frees all resources form the indy distributed ledger
func (r *Client) Close() error {
	result := C.indy_vdr_pool_close(C.int64_t(r.pool))
	if result != 0 {
		return fmt.Errorf("close indy pool failed: (Indy error code: [%v])", result)
	}

	return nil
}

type SubmitResponse struct {
	ErrorCode int
	Response  string
}

var submitRequestCh = make(chan SubmitResponse, 1)
var submitRequestLock = sync.Mutex{}

//export submitRequestCb
func submitRequestCb(cb_id C.CallbackId, err C.ErrorCode, response *C.char) {
	msg := SubmitResponse{
		ErrorCode: int(err),
		Response:  C.GoString(response),
	}

	submitRequestCh <- msg
}

//Submit is used to send prepared read requests to the ledger where the request parameter is the JSON-formatted payload.
func (r *Client) Submit(request []byte) (*ReadReply, error) {

	var cusreq C.int64_t
	cjson := C.CString(string(request))
	result := C.indy_vdr_build_custom_request(cjson, &cusreq)
	C.free(unsafe.Pointer(cjson))
	if result != 0 {
		var errMsg *C.char
		C.indy_vdr_get_current_error(&errMsg)
		defer C.free(unsafe.Pointer(errMsg))
		return nil, fmt.Errorf("invalid custom request: (Indy error code: [%s])", C.GoString(errMsg))
	}
	defer C.indy_vdr_request_free(cusreq)

	return r.submitReadRequest(cusreq)
}

//GetNym fetches the NYM transaction associated with a DID
func (r *Client) GetNym(did string) (*ReadReply, error) {
	var nymreq C.int64_t
	var none *C.char
	cdid := C.CString(did)
	result := C.indy_vdr_build_get_nym_request(none, cdid, &nymreq)
	C.free(unsafe.Pointer(cdid))
	if result != 0 {
		return nil, fmt.Errorf("invalid get nym request: (Indy error code: [%v])", result)
	}
	defer C.indy_vdr_request_free(nymreq)

	return r.submitReadRequest(nymreq)
}

//GetTxnAuthorAgreement fetches the current ledger Transaction Author Agreement
func (r *Client) GetTxnAuthorAgreement() (*ReadReply, error) {
	var taareq C.int64_t
	var none *C.char
	result := C.indy_vdr_build_get_txn_author_agreement_request(none, none, &taareq)
	if result != 0 {
		return nil, fmt.Errorf("invalid get taa request: (Indy error code: [%v])", result)
	}
	defer C.indy_vdr_request_free(taareq)

	return r.submitReadRequest(taareq)
}

//GetAcceptanceMethodList fetches the current ledger Acceptance Methods List (for the TAA)
func (r *Client) GetAcceptanceMethodList() (*ReadReply, error) {
	var amlreq C.int64_t
	var none *C.char
	var zero C.int64_t
	result := C.indy_vdr_build_get_acceptance_mechanisms_request(none, zero, none, &amlreq)
	if result != 0 {
		return nil, fmt.Errorf("invalid get aml request: (Indy error code: [%v])", result)
	}
	defer C.indy_vdr_request_free(amlreq)

	return r.submitReadRequest(amlreq)
}

//GetEndpoint fetches the registered endpoint for a DID
func (r *Client) GetEndpoint(did string) (*ReadReply, error) {
	return r.GetAttrib(did, "endpoint")
}

type RefreshResponse struct {
	ErrorCode int
}

var refreshCh = make(chan RefreshResponse, 1)
var refreshLock = sync.Mutex{}

//export refreshCb
func refreshCb(cb_id C.CallbackId, err C.ErrorCode) {
	msg := RefreshResponse{
		ErrorCode: int(err),
	}

	refreshCh <- msg
}

//RefreshPool retrieves the current pool transactions for the ledger
func (r *Client) RefreshPool() error {
	refreshLock.Lock()
	defer refreshLock.Unlock()

	result := C.indy_vdr_pool_refresh(C.int64_t(r.pool), C.refreshWrapper(C.refresh), 0)
	if result != 0 {
		var errMsg *C.char
		C.indy_vdr_get_current_error(&errMsg)
		defer C.free(unsafe.Pointer(errMsg))
		return fmt.Errorf("refresh pool failed: (Indy error code: [%v] %s)", result, C.GoString(errMsg))
	}

	res := <-refreshCh
	if res.ErrorCode > 0 {
		return fmt.Errorf("refresh pool error result: (Indy error code: [%v])", res.ErrorCode)
	}

	return nil
}

type StatusResponse struct {
	ErrorCode int
	Response  string
}

var statusCh = make(chan StatusResponse, 1)
var statusLock = sync.Mutex{}

//export statusCb
func statusCb(cb_id C.CallbackId, err C.ErrorCode, response *C.char) {
	msg := StatusResponse{
		ErrorCode: int(err),
		Response:  C.GoString(response),
	}

	statusCh <- msg
}

//GetPoolStatus fetches the current status and node list of the distributed ledger
func (r *Client) GetPoolStatus() (*PoolStatus, error) {
	statusLock.Lock()
	defer statusLock.Unlock()

	result := C.indy_vdr_pool_get_status(C.int64_t(r.pool), C.statusWrapper(C.status), 0)
	if result != 0 {
		return nil, fmt.Errorf("get pool status failed: (Indy error code: [%v])", result)
	}

	res := <-statusCh
	if res.ErrorCode > 0 {
		return nil, fmt.Errorf("error from pool status: (Indy error code: [%v])", res.ErrorCode)
	}

	ps := &PoolStatus{}
	err := json.Unmarshal([]byte(res.Response), ps)
	if err != nil {
		return nil, fmt.Errorf("unmarshaling pool status failed: %w", err)
	}

	return ps, nil
}

//GetAttrib fetches the attribute from the raw field of the provided DID
func (r *Client) GetAttrib(did, raw string) (*ReadReply, error) {
	attribreq := NewRawAttribRequest(did, raw, did)
	d, err := json.Marshal(attribreq)
	if err != nil {
		return nil, fmt.Errorf("marhsal indy attr request failed: (%w)", err)
	}

	response, err := r.Submit(d)
	if err != nil {
		return nil, fmt.Errorf("unable to submit indy get attr request. (%w)", err)
	}

	return response, nil

}

//GetSchema returns the schema definition defined by schemaID on the Indy distributed ledger
func (r *Client) GetSchema(schemaID string) (*ReadReply, error) {
	var schemareq C.int64_t
	var none *C.char
	cschema := C.CString(schemaID)
	result := C.indy_vdr_build_get_schema_request(none, cschema, &schemareq)
	C.free(unsafe.Pointer(cschema))
	if result != 0 {
		return nil, fmt.Errorf("invalid get schema request: (Indy error code: [%v])", result)
	}
	defer C.indy_vdr_request_free(schemareq)

	return r.submitReadRequest(schemareq)

}

//GetCredDef returns the credential definition defined by credDefID on the Indy distributed ledger
func (r *Client) GetCredDef(credDefID string) (*ReadReply, error) {
	var credDefReqNo C.int64_t
	var none *C.char
	cschema := C.CString(credDefID)
	result := C.indy_vdr_build_get_cred_def_request(none, cschema, &credDefReqNo)
	C.free(unsafe.Pointer(cschema))
	if result != 0 {
		return nil, fmt.Errorf("invalid get credential definition request: (Indy error code: [%v])", result)
	}
	defer C.indy_vdr_request_free(credDefReqNo)

	return r.submitReadRequest(credDefReqNo)

}

//GetAuthRules fetches all AUTH rules for the ledger
func (r *Client) GetAuthRules() (*ReadReply, error) {
	return r.GetTxnTypeAuthRule("", "", "")
}

//TODO:  figure out why "*" doesn't work as a wildcard for field
//GetTxnTypeAuthRule fetches the AUTH rule for a specific transaction type and action
func (r *Client) GetTxnTypeAuthRule(typ, action, field string) (*ReadReply, error) {
	var authReq *Request
	switch action {
	case AuthActionEdit:
		authReq = NewAuthEditRuleRequest(typ, field)
	case AuthActionAdd:
		authReq = NewAuthAddRuleRequest(typ, field)
	default:
		authReq = NewAuthRulesRequest()
	}

	d, err := json.Marshal(authReq)
	if err != nil {
		return nil, fmt.Errorf("marhsal indy auth rule request failed: (%w)", err)
	}

	response, err := r.Submit(d)
	if err != nil {
		return nil, fmt.Errorf("unable to submit indy auth rule request. (%w)", err)
	}

	return response, nil
}

func (r *Client) submitReadRequest(reqID C.int64_t) (*ReadReply, error) {
	submitRequestLock.Lock()
	defer submitRequestLock.Unlock()
	result := C.indy_vdr_pool_submit_request(C.int64_t(r.pool), reqID, C.submitRequestWrapper(C.submitRequest), 0)
	if result != 0 {
		var errMsg *C.char
		C.indy_vdr_get_current_error(&errMsg)
		defer C.free(unsafe.Pointer(errMsg))
		return nil, fmt.Errorf("unable to submit request: (Indy error code: [%v] %s)", result, C.GoString(errMsg))
	}
	res := <-submitRequestCh
	if res.ErrorCode > 0 {
		var errMsg *C.char
		C.indy_vdr_get_current_error(&errMsg)
		defer C.free(unsafe.Pointer(errMsg))
		return nil, fmt.Errorf("error from submitted request: (Indy error code: [%v] %s)", result, C.GoString(errMsg))
	}

	rply, err := parseReadReply(res.Response)
	if err != nil {
		return nil, err
	}

	return rply, nil
}

func (r *Client) submitWriteRequest(reqID C.int64_t) (*WriteReply, error) {
	submitRequestLock.Lock()
	defer submitRequestLock.Unlock()
	result := C.indy_vdr_pool_submit_request(C.int64_t(r.pool), reqID, C.submitRequestWrapper(C.submitRequest), 0)
	if result != 0 {
		var errMsg *C.char
		C.indy_vdr_get_current_error(&errMsg)
		defer C.free(unsafe.Pointer(errMsg))
		return nil, fmt.Errorf("unable to submit request: (Indy error code: [%v] %s)", result, C.GoString(errMsg))
	}
	res := <-submitRequestCh
	if res.ErrorCode > 0 {
		var errMsg *C.char
		C.indy_vdr_get_current_error(&errMsg)
		defer C.free(unsafe.Pointer(errMsg))
		return nil, fmt.Errorf("error from submitted request: (Indy error code: [%v] %s)", result, C.GoString(errMsg))
	}

	rply, err := parseWriteReply(res.Response)
	if err != nil {
		return nil, err
	}

	return rply, nil
}

//SubmitWrite is used to send prepared write requests to the ledger where the req parameter is the JSON-formatted payload.
//the signer defined a service capable of signing a message that is allowed to be written to the ledger.
func (r *Client) SubmitWrite(req *Request, signer Signer) (*WriteReply, error) {
	d, _ := json.MarshalIndent(req, " ", "")
	m := map[string]interface{}{}
	_ = json.Unmarshal(d, &m)

	ser, err := SerializeSignature(m)
	if err != nil {
		return nil, errors.Wrap(err, "unable to generate signature")
	}

	sig, err := signer.Sign([]byte(ser))
	if err != nil {
		return nil, errors.Wrap(err, "unable to sign write request")
	}

	req.Signature = base58.Encode(sig)
	request, err := json.MarshalIndent(req, " ", "")
	if err != nil {
		return nil, errors.Wrap(err, "unable to marshal write request")
	}

	var cusreq C.int64_t
	cjson := C.CString(string(request))
	result := C.indy_vdr_build_custom_request(cjson, &cusreq)
	C.free(unsafe.Pointer(cjson))
	if result != 0 {
		var errMsg *C.char
		C.indy_vdr_get_current_error(&errMsg)
		defer C.free(unsafe.Pointer(errMsg))
		return nil, fmt.Errorf("invalid custom writerequest: (Indy error code: [%s])", C.GoString(errMsg))
	}
	defer C.indy_vdr_request_free(cusreq)

	return r.submitWriteRequest(cusreq)
}
