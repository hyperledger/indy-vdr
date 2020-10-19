package vdr

import (
	"github.com/google/uuid"
)

type GetSchema struct {
	Operation `json:",inline"`
	Dest      string        `json:"dest"`
	Data      getSchemaData `json:"data"`
}

type getSchemaData struct {
	Name    string `json:"name"`
	Version string `json:"version"`
}

type Schema struct {
	Operation `json:",inline"`
	Dest      string     `json:"dest"`
	Data      SchemaData `json:"data"`
}

type SchemaData struct {
	ID        string   `json:"id"`
	SeqNo     uint32   `json:"seq_no"`
	Name      string   `json:"name"`
	Version   string   `json:"version"`
	AttrNames []string `json:"attr_names"`
}

func NewGetSchema(issuerDID, name, version, from string) *Request {
	return &Request{
		Operation: GetSchema{
			Operation: Operation{Type: GET_SCHEMA},
			Dest:      issuerDID,
			Data:      getSchemaData{Name: name, Version: version},
		},
		Identifier:      from,
		ProtocolVersion: protocolVersion,
		ReqID:           uuid.New().ID(),
	}
}

func NewSchema(issuerDID, name, version, from string, attrs []string) *Request {
	return &Request{
		Operation: Schema{
			Operation: Operation{Type: SCHEMA},
			Dest:      issuerDID,
			Data:      SchemaData{Name: name, Version: version, AttrNames: attrs},
		},
		Identifier:      from,
		ProtocolVersion: protocolVersion,
		ReqID:           uuid.New().ID(),
	}
}
