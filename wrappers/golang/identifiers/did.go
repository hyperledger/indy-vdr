package identifiers

import "C"
import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
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
	pubk, privk []byte
}

func (r *KeyPair) RawPublicKey() ed25519.PublicKey {
	return r.pubk
}

func (r *KeyPair) RawPrivateKey() ed25519.PrivateKey {
	return r.privk
}

func (r *KeyPair) PublicKey() string {
	return base58.Encode(r.pubk)
}

func (r *KeyPair) PrivateKey() string {
	return base58.Encode(r.privk)
}

type MyDIDInfo struct {
	DID        string
	Seed       string
	Cid        bool
	MethodName string
}

type DIDValue struct {
	DID    string
	Method string
}

func (r *DIDValue) String() string {
	if r.Method == "" {
		return fmt.Sprintf("did:%s", r.DID)
	}
	return fmt.Sprintf("did:%s:%s", r.Method, r.DID)
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

func (r *DID) AbbreviateVerkey() string {
	return AbbreviateVerkey(r.String(), r.Verkey)
}

func ParseDID(did string) *DIDValue {
	if DIDRegEx.MatchString(did) {
		p := DIDRegEx.FindStringSubmatch(did)
		return &DIDValue{
			DID:    p[2],
			Method: p[1],
		}

	}

	return &DIDValue{
		DID:    did,
		Method: "",
	}
}

func CreateDID(info *MyDIDInfo) (*DID, *KeyPair, error) {

	edseed, err := convertSeed(info.Seed)
	if err != nil {
		return nil, nil, errors.Wrap(err, "unable to get seed")
	}

	var pubkey ed25519.PublicKey
	var privkey ed25519.PrivateKey
	if len(edseed) == 0 {
		pubkey, privkey, err = ed25519.GenerateKey(rand.Reader)
		if err != nil {
			return nil, nil, errors.Wrap(err, "error generating keypair")
		}
	} else {
		privkey = ed25519.NewKeyFromSeed(edseed)
		pubkey = privkey.Public().(ed25519.PublicKey)
	}

	var did string
	if info.DID != "" {
		did = info.DID
	} else if info.Cid {
		did = base58.Encode(pubkey[0:16])
	} else {
		did = base58.Encode(pubkey)
	}

	out := &DID{
		DIDVal: DIDValue{
			DID:    did,
			Method: info.MethodName,
		},
		Verkey: base58.Encode(pubkey),
	}

	return out, &KeyPair{pubk: pubkey, privk: privkey}, nil

}

func convertSeed(seed string) ([]byte, error) {
	if seed == "" {
		return []byte{}, nil
	}

	if len(seed) == ed25519.SeedSize {
		return []byte(seed), nil
	}

	if strings.HasSuffix(seed, "=") {
		var out []byte
		c, err := base64.StdEncoding.Decode(out, []byte(seed))
		if err != nil || c != ed25519.SeedSize {
			return nil, errors.Wrap(err, "invalid base64 seed value")
		}
		return out, nil
	} else if len(seed) == 2*ed25519.SeedSize {
		var out []byte
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

	bdid, err := base58.Decode(didval.DID)
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
