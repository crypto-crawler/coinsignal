package utils

import (
	"context"
	"encoding/json"
	"log"

	"github.com/go-redis/redis/v8"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pojo"
)

type PriceUpdater struct {
	pubsub *redis.PubSub
	prices map[string]float64
}

func NewPriceUpdater(ctx context.Context, redis_url string) *PriceUpdater {
	rdb := redis.NewClient(&redis.Options{
		Addr: redis_url,
	})
	pubsub := rdb.Subscribe(ctx, config.REDIS_TOPIC_MARK_PRICE)

	updater := &PriceUpdater{
		pubsub: pubsub,
		prices: make(map[string]float64),
	}

	go updater.run()

	return updater
}

func (updater *PriceUpdater) GetPrice(currency string) float64 {
	price, _ := updater.prices[currency]
	return price
}

func (updater *PriceUpdater) Close() {
 updater.pubsub.Close()
}

func (updater *PriceUpdater) run() {
	ch := updater.pubsub.Channel()
	// Consume messages.
	for msg := range ch {
		var mark_prices []pojo.MarkPrice
		if err := json.Unmarshal([]byte(msg.Payload), &mark_prices); err != nil {
			log.Fatalln(err)
		} else {
			for _, x := range mark_prices {
				updater.prices[x.Currency] = x.Price
			}
		}
	}
}
