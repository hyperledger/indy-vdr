package vdr

import (
	"encoding/json"
	"testing"
)

func TestSerializeSignature(t *testing.T) {
	type args struct {
		value string
	}
	tests := []struct {
		name    string
		args    args
		want    string
		wantErr bool
	}{
		{
			name: "happy path",
			args: args{value: `{
			                "name": "John Doe",
			                "age": 43,
			                "operation": {
			                    "dest": 54
			                },
			                "phones": [
			                  "1234567",
			                  "2345678",
			                  {"rust": 5, "age": 1},
			                  3
			                ]
			            }`,
			},
			want:    `age:43|name:John Doe|operation:dest:54|phones:1234567,2345678,age:1|rust:5,3`,
			wantErr: false,
		},
		{
			name:    "dict with array",
			args:    args{value: `{"1": "a", "2": "b", "3": ["1", {"2": "k"}]}`},
			want:    "1:a|2:b|3:1,2:k",
			wantErr: false,
		},
		{
			name:    "ordered dict",
			args:    args{value: `{"2": "a", "3": "b", "1": ["1", {"2": "k"}]}`},
			want:    "1:1,2:k|2:a|3:b",
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := map[string]interface{}{}
			err := json.Unmarshal([]byte(tt.args.value), &m)
			if err != nil {
				t.Errorf("json.Unmarshal() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			got, err := serializeSignature(m, true, "")
			if (err != nil) != tt.wantErr {
				t.Errorf("SerializeSignature() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("SerializeSignature() got = %v | want %v", got, tt.want)
			}
		})
	}
}
