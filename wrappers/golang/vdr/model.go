package vdr

type PoolStatus struct {
	Root  string   `json:"mt_root"`
	Size  int      `json:"mt_size"`
	Nodes []string `json:"nodes"`
}
