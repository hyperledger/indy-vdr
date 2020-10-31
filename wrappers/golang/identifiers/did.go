package identifiers

import "C"
import (
	"bytes"
	"crypto/ed25519"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"regexp"
	"strings"

	"github.com/mr-tron/base58"
	"github.com/pkg/errors"
)

var DIDRegEx = regexp.MustCompile("^[a-z0-9]+:([a-z0-9]+):(.*)$")

type KeyPair struct {
	id   string
	pubk []byte
}

func NewKeyPair(pubk []byte, id string) *KeyPair {
	return &KeyPair{
		pubk: pubk,
		id:   id,
	}
}

func (r *KeyPair) RawPublicKey() ed25519.PublicKey {
	return r.pubk
}

func (r *KeyPair) ID() string {
	return r.id
}

func (r *KeyPair) PublicKey() string {
	return base58.Encode(r.pubk)
}

type MyDIDInfo struct {
	DID        string
	PublicKey  []byte
	Cid        bool
	MethodName string
}

type DIDValue struct {
	MethodSpecificID string
	Method           string
}

func (r *DIDValue) String() string {
	if r.Method == "" {
		return fmt.Sprintf("did:%s", r.MethodSpecificID)
	}
	return fmt.Sprintf("did:%s:%s", r.Method, r.MethodSpecificID)
}

func (r *DIDValue) Abbreviatable() bool {
	return r.Method == "sov" || r.Method == ""
}

type DID struct {
	DIDVal DIDValue
	Verkey string
}

func (r *DID) String() string {
	return r.DIDVal.String()
}

func (r *DID) MethodID() string {
	return r.DIDVal.MethodSpecificID
}

func (r *DID) AbbreviateVerkey() string {
	return AbbreviateVerkey(r.String(), r.Verkey)
}

func ParseDID(did string) DIDValue {
	if DIDRegEx.MatchString(did) {
		p := DIDRegEx.FindStringSubmatch(did)
		return DIDValue{
			MethodSpecificID: p[2],
			Method:           p[1],
		}

	}

	return DIDValue{
		MethodSpecificID: did,
		Method:           "",
	}
}

func CreateDID(info *MyDIDInfo) (*DID, error) {

	var did string
	if info.DID != "" {
		did = info.DID
	} else if info.Cid {
		did = base58.Encode(info.PublicKey[0:16])
	} else {
		did = base58.Encode(info.PublicKey)
	}

	out := &DID{
		DIDVal: DIDValue{
			MethodSpecificID: did,
			Method:           info.MethodName,
		},
		Verkey: base58.Encode(info.PublicKey),
	}

	return out, nil

}

func ConvertSeed(seed string) ([]byte, error) {
	if seed == "" {
		return []byte{}, nil
	}

	if len(seed) == ed25519.SeedSize {
		return []byte(seed), nil
	}

	if strings.HasSuffix(seed, "=") {
		out := make([]byte, ed25519.SeedSize)
		c, err := base64.StdEncoding.Decode(out, []byte(seed))
		if err != nil || c != ed25519.SeedSize {
			return nil, errors.Wrap(err, "invalid base64 seed value")
		}
		return out, nil
	} else if len(seed) == 2*ed25519.SeedSize {
		out := make([]byte, ed25519.SeedSize)
		_, err := hex.Decode(out, []byte(seed))
		if err != nil {
			return nil, errors.Wrap(err, "invalid hex seed value")
		}
		return out, nil
	}

	return []byte{}, nil
}

func AbbreviateVerkey(did, verkey string) string {
	didval := ParseDID(did)

	if !didval.Abbreviatable() {
		return verkey
	}

	bdid, err := base58.Decode(didval.MethodSpecificID)
	if err != nil {
		return verkey
	}

	bverkey, err := base58.Decode(verkey)
	if err != nil {
		return verkey
	}

	if !bytes.Equal(bdid, bverkey[0:16]) {
		return verkey
	}

	abbrev := base58.Encode(bverkey[16:])
	return fmt.Sprintf("~%s", abbrev)
}
