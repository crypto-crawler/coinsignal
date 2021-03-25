package main

import (
	"context"
	"log"
	"os"

	"github.com/buger/jsonparser"
	"github.com/go-redis/redis/v8"
	"github.com/gorilla/websocket"
	"github.com/soulmachine/coinsignal/config"
)

func main() {
	ctx := context.Background()

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}
	rdb := redis.NewClient(&redis.Options{
		Addr: redis_url,
	})

	client, _, err := websocket.DefaultDialer.Dial("wss://fstream.binance.com/ws/ethusdt@markPrice", nil)
	if err != nil {
		log.Fatal(err)
	}
	for {
		_, message, _ := client.ReadMessage()
		price, _, _, _ := jsonparser.Get(message, "p")
		err := rdb.Set(ctx, config.REDIS_TOPIC_ETH_PRICE, price, 0).Err()
		if err != nil {
			log.Fatal(err)
		}
	}
	// rdb.Close()
}
