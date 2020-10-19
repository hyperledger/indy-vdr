package vdr

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestNewSchema(t *testing.T) {

	attrs := []string{"name", "expiration"}
	cd := NewSchema("did:xyz:123456", "test-schema", "1.2", "did:xyz:456789", attrs)

	require.Equal(t, cd.Operation, Schema{
		Operation: Operation{Type: SCHEMA},
		Dest:      "did:xyz:123456",
		Data:      SchemaData{Name: "test-schema", Version: "1.2", AttrNames: attrs},
	})
	require.Equal(t, cd.ProtocolVersion, 2)
	require.Equal(t, cd.Identifier, "did:xyz:456789")
}

func TestNewGetSchema(t *testing.T) {

	cd := NewGetSchema("did:xyz:123456", "test-schema", "1.2", "did:xyz:456789")

	require.Equal(t, cd.Operation, GetSchema{
		Operation: Operation{Type: GET_SCHEMA},
		Dest:      "did:xyz:123456",
		Data:      getSchemaData{Name: "test-schema", Version: "1.2"},
	})
	require.Equal(t, cd.ProtocolVersion, 2)

}
