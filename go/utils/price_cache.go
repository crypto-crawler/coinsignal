package utils

import (
	"context"
	"log"
	"strconv"
	"time"

	"github.com/go-redis/redis/v8"
	"github.com/soulmachine/coinsignal/config"
)

var HOT_CURRENCIES = []string{"BTC", "ETH", "BNB", "ADA", "DOT", "XRP", "UNI", "LTC", "THETA", "LINK", "BCH", "XLM", "KLAY", "FIL"}

type PriceCache struct {
	client *redis.Client
	ctx    context.Context
	prices map[string]float64
}

func NewPriceCache(ctx context.Context, redis_url string) *PriceCache {
	client := NewRedisClient(redis_url)

	cache := &PriceCache{
		client: client,
		ctx:    ctx,
		prices: make(map[string]float64),
	}

	go cache.update()

	return cache
}

func (cache *PriceCache) WaitUntilReady() {
	for {
		if cache.isReady() {
			break
		} else {
			log.Println("price cache is not ready yet")
			time.Sleep(3 * time.Second)
		}
	}
}

func (cache *PriceCache) GetPrice(currency string) float64 {
	price := cache.prices[currency]
	return price
}

func (cache *PriceCache) Close() {
	cache.client.Close()
}

// retrieves every 3 seconds
func (cache *PriceCache) update() {
	for {
		m, err := cache.client.HGetAll(cache.ctx, config.REDIS_TOPIC_CURRENCY_PRICE).Result()

		if err == nil {
			for k, v := range m {
				price, _ := strconv.ParseFloat(v, 64)
				cache.prices[k] = price
			}
		}

		time.Sleep(3 * time.Second)
	}
}

func (cache *PriceCache) isReady() bool {
	for _, currency := range HOT_CURRENCIES {
		if _, ok := cache.prices[currency]; !ok {
			return false
		}
	}
	return true
}
