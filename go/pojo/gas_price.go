package pojo

import "encoding/json"

// price in USD
type GasPrice struct {
	Rapid     float64 `json:"rapid"`
	Fast      float64 `json:"fast"`
	Standard  float64 `json:"standard"`
	Slow      float64 `json:"slow"`
	Timestamp int64   `json:"timestamp"`
}

// price in Wei
type gasPriceMsg struct {
	Rapid     uint64  `json:"rapid"`
	Fast      uint64  `json:"fast"`
	Standard  uint64  `json:"standard"`
	Slow      uint64  `json:"slow"`
	Timestamp int64   `json:"timestamp"`
	PriceUSD  float64 `json:"priceUSD"`
}

// wei to USD
func fromWei(wei uint64, eth_price float64) float64 {
	return float64(wei) / 1000000000000000000 * 21000 * eth_price
}

func (gas_price *GasPrice) FromGasPriceMsg(text string) error {
	msg := gasPriceMsg{}
	err := json.Unmarshal([]byte(text), &msg)
	if err != nil {
		return err
	}

	gas_price.Rapid = fromWei(msg.Rapid, msg.PriceUSD)
	gas_price.Fast = fromWei(msg.Fast, msg.PriceUSD)
	gas_price.Standard = fromWei(msg.Standard, msg.PriceUSD)
	gas_price.Slow = fromWei(msg.Slow, msg.PriceUSD)
	gas_price.Timestamp = msg.Timestamp
	return nil
}
