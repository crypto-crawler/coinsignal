package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"strconv"
	"strings"

	"github.com/buger/jsonparser"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pojo"
	"github.com/soulmachine/coinsignal/pubsub"
	"github.com/soulmachine/coinsignal/utils"
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
	utils.WaitRedis(ctx, redis_url)

	rdb := utils.NewRedisClient(redis_url)
	publisher := pubsub.NewPublisher(ctx, redis_url)

	pubsub := rdb.Subscribe(ctx,
		config.REDIS_TOPIC_FUNDING_RATE,
	)

	// Consume messages.
	for msg := range pubsub.Channel() {
		raw_msg := pojo.CarbonbotMessage{}
		json.Unmarshal([]byte(msg.Payload), &raw_msg)
		if raw_msg.Exchange != "binance" {
			continue
		}

		raw_json := []byte(raw_msg.Json)
		data, _, _, _ := jsonparser.Get(raw_json, "data")

		var mark_prices_raw []MarkPriceRaw
		if err := json.Unmarshal(data, &mark_prices_raw); err != nil {
			panic(err)
		}

		for _, mark_price_raw := range mark_prices_raw {
			var currency string
			if strings.HasSuffix(mark_price_raw.Symbol, "USD_PERP") {
				currency = mark_price_raw.Symbol[:len(mark_price_raw.Symbol)-8]
			} else if strings.HasSuffix(mark_price_raw.Symbol, "USDT") || strings.HasSuffix(mark_price_raw.Symbol, "BUSD") {
				currency = mark_price_raw.Symbol[:len(mark_price_raw.Symbol)-4]
			} else {
				continue
			}
			price, _ := strconv.ParseFloat(mark_price_raw.Price, 64)

			currency_price := pojo.CurrencyPrice{
				Currency: currency,
				Price:    price,
			}

			json_bytes, _ := json.Marshal(currency_price)
			publisher.Publish(config.REDIS_TOPIC_CURRENCY_PRICE_CHANNEL, string(json_bytes))
		}
	}

	pubsub.Close()
	publisher.Close()
}
