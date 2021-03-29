package pojo

// price in USD
type GasPrice struct {
	Rapid     float64 `json:"rapid"`
	Fast      float64 `json:"fast"`
	Standard  float64 `json:"standard"`
	Slow      float64 `json:"slow"`
	Timestamp int64   `json:"timestamp"`
}
