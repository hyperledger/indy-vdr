package vdr

import (
	"github.com/pkg/errors"
)

// FIXME: Expose optional diddoc_content and version field on did:indy compliant ledgers
func (r *Client) CreateNym(did, verkey, role, from string, signer Signer) error {
	nymRequest := NewNym(did, verkey, from, role)

	_, err := r.SubmitWrite(nymRequest, signer)
	if err != nil {
		return err
	}

	return nil
}

func (r *Client) CreateAttrib(did, from string, data map[string]interface{}, signer Signer) error {
	rawAttrib := NewRawAttrib(did, from, data)

	_, err := r.SubmitWrite(rawAttrib, signer)
	if err != nil {
		return err
	}

	return nil
}

func (r *Client) SetEndpoint(did, from string, ep string, signer Signer) error {
	m := map[string]interface{}{"endpoint": map[string]interface{}{"endpoint": ep}}
	return r.CreateAttrib(did, from, m, signer)
}

func (r *Client) CreateSchema(issuerDID, name, version string, attrs []string, signer Signer) (string, error) {
	rawSchema := NewSchema(issuerDID, name, version, issuerDID, attrs)

	resp, err := r.SubmitWrite(rawSchema, signer)
	if err != nil {
		return "", errors.Wrap(err, "unable to create attrib")
	}

	return resp.TxnMetadata.TxnID, nil
}
