package pojo

type BlockHeader struct {
	Number    float64 `json:"number"`
	Miner     string  `json:"miner"`
	GasLimit  float64 `json:"gasLimit"`
	GasUsed   float64 `json:"gasUsed"`
	Reward    float64 `json:"reward"`
	RewardUSD float64 `json:"reward_usd"`
	Slow      float64 `json:"slow"`
	Timestamp int64   `json:"timestamp"`
}
