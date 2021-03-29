package main

import (
	"context"
	"encoding/json"
	"log"
	"os"

	"github.com/gorilla/websocket"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pojo"
	"github.com/soulmachine/coinsignal/pubsub"
	"github.com/soulmachine/coinsignal/utils"
)

// price in Wei
type GasPriceMsg struct {
	Rapid    uint64 `json:"rapid"`
	Fast     uint64 `json:"fast"`
	Standard uint64 `json:"standard"`
	Slow     uint64 `json:"slow"`
}

type Data struct {
	GasPrices GasPriceMsg `json:"gasPrices"`
	Timestamp int64       `json:"timestamp"`
}

type WebsocketMsg struct {
	Type string `json:"type"`
	Data Data   `json:"data"`
}

// wei to USD
func fromWei(wei uint64, eth_price float64) float64 {
	return float64(wei) / 1000000000000000000 * 21000 * eth_price
}

func main() {
	ctx := context.Background()

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}

	priceCache := utils.NewPriceCache(ctx, redis_url)
	priceCache.WaitUntilReady()
	publisher := pubsub.NewPublisher(ctx, redis_url)

	// see https://taichi.network/#gasnow
	client, _, err := websocket.DefaultDialer.Dial("wss://www.gasnow.org/ws", nil)
	if err != nil {
		log.Fatal(err)
	}
	for {
		_, message, _ := client.ReadMessage()
		ws_msg := WebsocketMsg{}
		json.Unmarshal(message, &ws_msg)

		ethPrice := priceCache.GetPrice("ETH")

		gas_price := &pojo.GasPrice{
			Rapid:     fromWei(ws_msg.Data.GasPrices.Rapid, ethPrice),
			Fast:      fromWei(ws_msg.Data.GasPrices.Fast, ethPrice),
			Standard:  fromWei(ws_msg.Data.GasPrices.Standard, ethPrice),
			Slow:      fromWei(ws_msg.Data.GasPrices.Slow, ethPrice),
			Timestamp: ws_msg.Data.Timestamp,
		}
		json_txt, _ := json.Marshal(&gas_price)

		if ethPrice > 0.0 {
			publisher.Publish(config.REDIS_TOPIC_ETH_GAS_PRICE, string(json_txt))
		}
	}

	// publisher.Close()
	// priceCache.Close()
}
