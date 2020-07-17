package vdr

import (
	"reflect"
	"testing"
)

func TestSortedMap(t *testing.T) {
	type fields struct {
		m map[string]interface{}
	}
	tests := []struct {
		name   string
		fields fields
		want   []string
	}{
		{
			name: "basic keys",
			fields: fields{
				m: map[string]interface{}{
					"y": "1",
					"z": "2",
					"a": "3",
				},
			},
			want: []string{"a", "y", "z"},
		},
		{
			name: "mixed case keys",
			fields: fields{
				m: map[string]interface{}{
					"y": "1",
					"Z": "2",
					"z": "3",
					"a": "3",
				},
			},
			want: []string{"Z", "a", "y", "z"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := NewSortedMap(tt.fields.m)
			if !reflect.DeepEqual(r.Keys, tt.want) {
				t.Errorf("Get() = %v, want %v", r.Keys, tt.want)
			}
		})
	}
}
