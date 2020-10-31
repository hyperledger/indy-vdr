package vdr

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestClaimDefData_PKey(t *testing.T) {
	type fields struct {
		ID         string
		Primary    map[string]interface{}
		Revocation map[string]interface{}
	}
	tests := []struct {
		name   string
		fields fields
		want   string
	}{
		{
			name: "get primary key",
			fields: fields{
				Primary: map[string]interface{}{"test": 123},
			},
			want: `{"test":123}`,
		},
		{
			name: "get empty revocation key",
			fields: fields{
				Revocation: nil,
			},
			want: `null`,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &ClaimDefData{
				ID:         tt.fields.ID,
				Primary:    tt.fields.Primary,
				Revocation: tt.fields.Revocation,
			}
			if got := r.PKey(); got != tt.want {
				t.Errorf("PKey() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestClaimDefData_RKey(t *testing.T) {
	type fields struct {
		ID         string
		Primary    map[string]interface{}
		Revocation map[string]interface{}
	}
	tests := []struct {
		name   string
		fields fields
		want   string
	}{
		{
			name: "get revocation key",
			fields: fields{
				Revocation: map[string]interface{}{"test": 123},
			},
			want: `{"test":123}`,
		},
		{
			name: "get empty revocation key",
			fields: fields{
				Revocation: nil,
			},
			want: `null`,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &ClaimDefData{
				ID:         tt.fields.ID,
				Primary:    tt.fields.Primary,
				Revocation: tt.fields.Revocation,
			}
			if got := r.RKey(); got != tt.want {
				t.Errorf("RKey() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestClaimDefData_UnmarshalReadReply(t *testing.T) {
	type args struct {
		rply *ReadReply
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "success unmarshal",
			args: args{
				rply: &ReadReply{
					Data: map[string]interface{}{
						"primary":    map[string]interface{}{"test": "abc"},
						"revocation": map[string]interface{}{"revoc": 123},
					},
				},
			},
			wantErr: false,
		},
		{
			name: "missing primary",
			args: args{
				rply: &ReadReply{
					Data: map[string]interface{}{
						"revocation": map[string]interface{}{"revoc": 123},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "invalid data",
			args: args{
				rply: &ReadReply{
					Data: "",
				},
			},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &ClaimDefData{}
			if err := r.UnmarshalReadReply(tt.args.rply); (err != nil) != tt.wantErr {
				t.Errorf("UnmarshalReadReply() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestNewClaimDef(t *testing.T) {

	primary := map[string]interface{}{}
	revocation := map[string]interface{}{}
	cd := NewClaimDef("did:xyz:123456", 246, primary, revocation)

	require.Equal(t, cd.Operation, ClaimDef{
		Operation:     Operation{Type: CLAIM_DEF},
		SignatureType: "CL",
		Ref:           246,
		Tag:           "default",
		Data:          ClaimDefData{Primary: primary, Revocation: revocation},
	})
	require.Equal(t, cd.ProtocolVersion, 2)
	require.Equal(t, cd.Identifier, "did:xyz:123456")
}

func TestNewGetClaimDef(t *testing.T) {

	cd := NewGetClaimDef("did:xyz:123456", 246)

	require.Equal(t, cd.Operation, GetClaimDef{
		Operation:     Operation{Type: GET_CLAIM_DEF},
		SignatureType: "CL",
		Ref:           246,
		Origin:        "did:xyz:123456",
		Tag:           "default",
	})
	require.Equal(t, cd.ProtocolVersion, 2)

}
