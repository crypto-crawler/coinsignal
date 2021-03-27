package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"strconv"

	"github.com/gorilla/websocket"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pojo"
	"github.com/soulmachine/coinsignal/pubsub"
)

type MarkPriceRaw struct {
	Symbol string `json:"s"`
	Price  string `json:"p"`
}

func main() {
	ctx := context.Background()

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}
	publisher := pubsub.NewPublisher(ctx, redis_url)

	// https://binance-docs.github.io/apidocs/futures/en/#mark-price-stream
	client, _, err := websocket.DefaultDialer.Dial("wss://fstream.binance.com/ws/!markPrice@arr", nil)
	if err != nil {
		log.Fatal(err)
	}
	for {
		_, bytes, _ := client.ReadMessage()
		var mark_prices_raw []MarkPriceRaw
		if err := json.Unmarshal(bytes, &mark_prices_raw); err != nil {
			panic(err)
		}

		var mark_prices []pojo.MarkPrice
		for _, mark_price_raw := range mark_prices_raw {
			currency := mark_price_raw.Symbol[:len(mark_price_raw.Symbol)-4]
			price, _ := strconv.ParseFloat(mark_price_raw.Price, 64)
			mark_price := pojo.MarkPrice{
				Currency: currency,
				Price:    price,
			}
			mark_prices = append(mark_prices, mark_price)
		}

		json_bytes, _ := json.Marshal(mark_prices)
		publisher.Publish(config.REDIS_TOPIC_MARK_PRICE, string(json_bytes))
	}
	// publisher.Close()
}
