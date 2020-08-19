package identifiers

import (
	"reflect"
	"testing"
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
					Seed:       "b2352b32947e188eb72871093ac6217e",
					Cid:        true,
					MethodName: "sov",
				},
			},
			want: &DID{
				DIDVal: DIDValue{
					DID:    "WvRwKqxFLtJ3YbhmHZBpmy",
					Method: "sov",
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
					Seed:       "b2352b32947e188eb72871093ac6217e",
					Cid:        false,
					MethodName: "",
				},
			},
			want: &DID{
				DIDVal: DIDValue{
					DID:    "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
					Method: "",
				},
				Verkey: "HJsMyfABm7gmPse8QzgUePRwTbQRyALgeZudJuYbYmro",
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, _, err := CreateDID(tt.args.info)
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
				DID:    tt.fields.DID,
				Method: tt.fields.Method,
			}
			if got := r.Abbreviatable(); got != tt.want {
				t.Errorf("Abbreviatable() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDIDValue_String(t *testing.T) {
	type fields struct {
		DID    string
		Method string
	}
	tests := []struct {
		name   string
		fields fields
		want   string
	}{
		// TODO: Add test cases.
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &DIDValue{
				DID:    tt.fields.DID,
				Method: tt.fields.Method,
			}
			if got := r.String(); got != tt.want {
				t.Errorf("String() = %v, want %v", got, tt.want)
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
		// TODO: Add test cases.
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
					DID:    "WvRwKqxFLtJ3YbhmHZBpmy",
					Method: "sov",
				},
				Verkey: "",
			},
			want: "did:sov:WvRwKqxFLtJ3YbhmHZBpmy",
		},
		{
			name: "no method",
			fields: fields{
				DIDVal: DIDValue{
					DID:    "WvRwKqxFLtJ3YbhmHZBpmy",
					Method: "",
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
		want *DIDValue
	}{
		{
			name: "test valid did",
			args: args{
				did: "did:ioe:PBu1XhbSQCdaeEKuJVFTi4",
			},
			want: &DIDValue{
				DID:    "PBu1XhbSQCdaeEKuJVFTi4",
				Method: "ioe",
			},
		},
		{
			name: "test valid did",
			args: args{
				did: "PBu1XhbSQCdaeEKuJVFTi4",
			},
			want: &DIDValue{
				DID:    "PBu1XhbSQCdaeEKuJVFTi4",
				Method: "",
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
