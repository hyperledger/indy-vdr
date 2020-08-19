package vdr

import (
	"sort"
)

type SortedMap struct {
	m    map[string]interface{}
	Keys []string
}

func NewSortedMap(m map[string]interface{}) *SortedMap {
	k := make([]string, len(m))
	i := 0
	for key, _ := range m {
		k[i] = key
		i++
	}
	sort.Strings(k)
	return &SortedMap{m: m, Keys: k}
}

func (r *SortedMap) Get(k string) interface{} {
	return r.m[k]
}
