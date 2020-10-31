package identifiers

import (
	"crypto/ed25519"
	"reflect"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestAbbreviateVerkey(t *testing.T) {
	type args struct {
		did    string
		verkey string
	}
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			name: "a key that can abbreviated",
			args: args{
				did:    "did:sov:WvRwKqxFLtJ3YbhmHZBpmy",
				verkey: "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
			},
			want: "~TkfxnTVB6SBQNAdtNHJHef",
		},
		{
			name: "a non-abbreviatable key",
			args: args{
				did:    "did:peer:WvRwKqxFLtJ3YbhmHZBTTL",
				verkey: "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
			},
			want: "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := AbbreviateVerkey(tt.args.did, tt.args.verkey); got != tt.want {
				t.Errorf("AbbreviateVerkey() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCreateDID(t *testing.T) {
	type args struct {
		info *MyDIDInfo
		seed string
	}
	tests := []struct {
		name    string
		args    args
		want    *DID
		wantErr bool
	}{
		{
			name: "with seed",
			args: args{
				info: &MyDIDInfo{
					DID:        "",
					Cid:        true,
					MethodName: "sov",
				},
				seed: "b2352b32947e188eb72871093ac6217e",
			},
			want: &DID{
				DIDVal: DIDValue{
					MethodSpecificID: "WvRwKqxFLtJ3YbhmHZBpmy",
					Method:           "sov",
				},
				Verkey: "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
			},
			wantErr: false,
		},
		{
			name: "without cid and no method",
			args: args{
				info: &MyDIDInfo{
					DID:        "",
					Cid:        false,
					MethodName: "",
				},
				seed: "b2352b32947e188eb72871093ac6217e",
			},
			want: &DID{
				DIDVal: DIDValue{
					MethodSpecificID: "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
					Method:           "",
				},
				Verkey: "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {

			if tt.args.seed != "" {
				edseed, err := ConvertSeed(tt.args.seed)
				require.NoError(t, err)
				privkey := ed25519.NewKeyFromSeed(edseed)
				tt.args.info.PublicKey = privkey.Public().(ed25519.PublicKey)
			}
			got, err := CreateDID(tt.args.info)
			if (err != nil) != tt.wantErr {
				t.Errorf("CreateDID() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("CreateDID() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDIDValue_Abbreviatable(t *testing.T) {
	type fields struct {
		DID    string
		Method string
	}
	tests := []struct {
		name   string
		fields fields
		want   bool
	}{
		{
			name: "with sov did",
			fields: fields{
				DID:    "WvRwKqxFLtJ3YbhmHZBpmy",
				Method: "sov",
			},
			want: true,
		},
		{
			name: "with empty method",
			fields: fields{
				DID:    "WvRwKqxFLtJ3YbhmHZBpmy",
				Method: "",
			},
			want: true,
		},
		{
			name: "with other method",
			fields: fields{
				DID:    "WvRwKqxFLtJ3YbhmHZBpmy",
				Method: "peer",
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &DIDValue{
				MethodSpecificID: tt.fields.DID,
				Method:           tt.fields.Method,
			}
			if got := r.Abbreviatable(); got != tt.want {
				t.Errorf("Abbreviatable() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDID_AbbreviateVerkey(t *testing.T) {
	type fields struct {
		DIDVal DIDValue
		Verkey string
	}
	tests := []struct {
		name   string
		fields fields
		want   string
	}{
		{
			name: "parse valid key",
			fields: fields{
				DIDVal: DIDValue{
					MethodSpecificID: "PH84KtiPeumMw3HbXWMPjP",
					Method:           "sov",
				},
				Verkey: "D9FWnVELJTifG4aycimQSaCbLK4Y6h67p1W5M83uZ7c1",
			},
			want: "~3EkLdeUVCf8j9fxVd2S6wX",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &DID{
				DIDVal: tt.fields.DIDVal,
				Verkey: tt.fields.Verkey,
			}
			if got := r.AbbreviateVerkey(); got != tt.want {
				t.Errorf("AbbreviateVerkey() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDID_String(t *testing.T) {
	type fields struct {
		DIDVal DIDValue
		Verkey string
	}
	tests := []struct {
		name   string
		fields fields
		want   string
	}{
		{
			name: "test formatting",
			fields: fields{
				DIDVal: DIDValue{
					MethodSpecificID: "WvRwKqxFLtJ3YbhmHZBpmy",
					Method:           "sov",
				},
				Verkey: "",
			},
			want: "did:sov:WvRwKqxFLtJ3YbhmHZBpmy",
		},
		{
			name: "no method",
			fields: fields{
				DIDVal: DIDValue{
					MethodSpecificID: "WvRwKqxFLtJ3YbhmHZBpmy",
					Method:           "",
				},
				Verkey: "",
			},
			want: "did:WvRwKqxFLtJ3YbhmHZBpmy",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &DID{
				DIDVal: tt.fields.DIDVal,
				Verkey: tt.fields.Verkey,
			}
			if got := r.String(); got != tt.want {
				t.Errorf("String() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParseDID(t *testing.T) {
	type args struct {
		did string
	}
	tests := []struct {
		name string
		args args
		want DIDValue
	}{
		{
			name: "test valid did",
			args: args{
				did: "did:ioe:PBu1XhbSQCdaeEKuJVFTi4",
			},
			want: DIDValue{
				MethodSpecificID: "PBu1XhbSQCdaeEKuJVFTi4",
				Method:           "ioe",
			},
		},
		{
			name: "test valid did",
			args: args{
				did: "PBu1XhbSQCdaeEKuJVFTi4",
			},
			want: DIDValue{
				MethodSpecificID: "PBu1XhbSQCdaeEKuJVFTi4",
				Method:           "",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := ParseDID(tt.args.did); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("ParseDID() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestConvertSeed(t *testing.T) {
	type args struct {
		seed string
	}
	tests := []struct {
		name    string
		args    args
		want    []byte
		wantErr bool
	}{
		{
			name: "straight key",
			args: args{
				seed: "b2352b32947e188eb72871093ac6217e",
			},
			want:    []byte("b2352b32947e188eb72871093ac6217e"),
			wantErr: false,
		},
		{
			name: "base64",
			args: args{
				seed: "YjIzNTJiMzI5NDdlMTg4ZWI3Mjg3MTA5M2FjNjIxN2U=",
			},
			want:    []byte("b2352b32947e188eb72871093ac6217e"),
			wantErr: false,
		},
		{
			name: "hex",
			args: args{
				seed: "6232333532623332393437653138386562373238373130393361633632313765",
			},
			want:    []byte("b2352b32947e188eb72871093ac6217e"),
			wantErr: false,
		},
		{
			name: "bad base64",
			args: args{
				seed: "12=",
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "bad hex",
			args: args{
				seed: "62323335326233323934376531383865623732383731303933616336323137GG",
			},
			want:    nil,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ConvertSeed(tt.args.seed)
			if (err != nil) != tt.wantErr {
				t.Errorf("ConvertSeed() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("ConvertSeed() got = %v, want %v", string(got), tt.want)
			}
		})
	}
}
