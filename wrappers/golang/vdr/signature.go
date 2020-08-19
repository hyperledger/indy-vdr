package vdr

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"strings"

	"github.com/pkg/errors"
)

type Signer interface {
	Sign([]byte) ([]byte, error)
}

func SerializeSignature(value map[string]interface{}) (string, error) {
	m, ok := value["operation"].(map[string]interface{})
	if !ok {
		return "", errors.New("missing operation")
	}
	typ, _ := m["type"].(string)

	return serializeSignature(value, true, typ)
}

func serializeSignature(value interface{}, isTop bool, typ string) (string, error) {
	switch v := value.(type) {
	case bool:
		if v {
			return "True", nil
		}
		return "False", nil
	case string:
		return v, nil
	case float64:
		d, _ := json.Marshal(v)
		return string(d), nil
	case []interface{}:
		var err error
		out := make([]string, len(v))
		for i, val := range v {
			out[i], err = serializeSignature(val, false, typ)
			if err != nil {
				return "", err
			}
		}
		return strings.Join(out, ","), nil
	case map[string]interface{}:
		var result string
		var inMiddle = false
		m := NewSortedMap(v)
		for _, key := range m.Keys {
			val := m.Get(key)
			if isTop && (key == "signature" || key == "fees" || key == "signatures") {
				continue
			}

			if inMiddle {
				result += "|"
			}

			if (typ == ATTRIB || typ == GET_ATTR) && (key == "raw" || key == "hash" || key == "enc") {
				sh := sha256.Sum256([]byte(val.(string)))
				val = hex.EncodeToString(sh[:])
			}

			nest, err := serializeSignature(val, false, typ)
			if err != nil {
				return "", errors.Wrap(err, "error seializing map value")
			}
			result = strings.Join([]string{result, key, ":", nest}, "")
			inMiddle = true
		}
		return result, nil
	}

	return "", nil
}
