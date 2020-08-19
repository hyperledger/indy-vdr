package crypto

import (
	"crypto/ed25519"
)

type Ed25519Signer struct {
	publicKey  ed25519.PublicKey
	privateKey ed25519.PrivateKey
}

func NewSigner(pubkey ed25519.PublicKey, privkey ed25519.PrivateKey) *Ed25519Signer {
	return &Ed25519Signer{
		publicKey:  pubkey,
		privateKey: privkey,
	}
}

func (r *Ed25519Signer) Sign(d []byte) ([]byte, error) {
	return ed25519.Sign(r.privateKey, d), nil
}
