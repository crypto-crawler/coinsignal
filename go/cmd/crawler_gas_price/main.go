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
	Timestamp uint64      `json:"timestamp"`
}

type WebsocketMsg struct {
	Type string `json:"type"`
	Data Data   `json:"data"`
}

// price in USD
type GasPrice struct {
	Rapid     float64 `json:"rapid"`
	Fast      float64 `json:"fast"`
	Standard  float64 `json:"standard"`
	Slow      float64 `json:"slow"`
	Timestamp uint64  `json:"timestamp"`
}

// wei to USD
func fromWei(wei uint64, eth_price float64) float64 {
	return float64(wei) / 1000000000000000000 * 21000 * eth_price
}

var ethPrice = 0.0

func onMsg(msg string) {
	var mark_prices []pojo.MarkPrice
	if err := json.Unmarshal([]byte(msg), &mark_prices); err != nil {
		panic(err)
	}
	for _, x := range mark_prices {
		if x.Currency == "ETH" {
			ethPrice = x.Price
		}
	}
}

func main() {
	ctx := context.Background()

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}

	subscriber := pubsub.NewSubscriber(ctx, redis_url, config.REDIS_TOPIC_MARK_PRICE, onMsg)
	go subscriber.Run()

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

		gas_price := &GasPrice{
			Rapid:     fromWei(ws_msg.Data.GasPrices.Rapid, ethPrice),
			Fast:      fromWei(ws_msg.Data.GasPrices.Fast, ethPrice),
			Standard:  fromWei(ws_msg.Data.GasPrices.Standard, ethPrice),
			Slow:      fromWei(ws_msg.Data.GasPrices.Slow, ethPrice),
			Timestamp: ws_msg.Data.Timestamp,
		}
		json_txt, _ := json.Marshal(&gas_price)

		if ethPrice > 0 {
			publisher.Publish(config.REDIS_TOPIC_ETH_GAS_PRICE, string(json_txt))
		}
	}

	// publisher.Close()
	// subscriber.Close()
}
