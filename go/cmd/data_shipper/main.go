package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/go-redis/redis/v8"
	influxdb2 "github.com/influxdata/influxdb-client-go/v2"
	"github.com/influxdata/influxdb-client-go/v2/api"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pojo"
	"github.com/soulmachine/coinsignal/utils"
)

// read data from Redis channels and write to Influxdb
func main() {
	ctx := context.Background()

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}
	rdb := utils.NewRedisClient(redis_url)
	utils.WaitRedis(ctx, redis_url)

	influxdb_url := os.Getenv("INFLUXDB_URL")
	if len(influxdb_url) == 0 {
		log.Fatal("The INFLUXDB_URL environment variable is empty")
	}
	influxdb_token := os.Getenv("INFLUXDB_TOKEN")
	if len(influxdb_token) == 0 {
		log.Fatal("The INFLUXDB_TOKEN environment variable is empty")
	}
	influxdb_org := os.Getenv("INFLUXDB_ORG")
	if len(influxdb_token) == 0 {
		log.Fatal("The INFLUXDB_ORG environment variable is empty")
	}
	influxdb_bucket := os.Getenv("INFLUXDB_BUCKET")
	if len(influxdb_token) == 0 {
		log.Fatal("The INFLUXDB_BUCKET environment variable is empty")
	}
	client := influxdb2.NewClientWithOptions(influxdb_url, influxdb_token,
		influxdb2.DefaultOptions().SetBatchSize(32))
	utils.WaitInfluxDB(ctx, client)

	// Get non-blocking write client
	writeAPI := client.WriteAPI(influxdb_org, influxdb_bucket)

	pubsub := rdb.Subscribe(ctx,
		config.REDIS_TOPIC_ETH_GAS_PRICE,
		config.REDIS_TOPIC_ETH_BLOCK_HEADER,
		config.REDIS_TOPIC_CANDLESTICK_EXT,
		config.REDIS_TOPIC_CMC_GLOBAL_METRICS,
	)

	// Consume messages.
	for msg := range pubsub.Channel() {
		handleMessage(msg, writeAPI)
	}

	pubsub.Close()
	rdb.Close()
}

func handleMessage(msg *redis.Message, writeAPI api.WriteAPI) {
	switch msg.Channel {
	case config.REDIS_TOPIC_ETH_GAS_PRICE:
		{
			gas_price := pojo.GasPrice{}
			json.Unmarshal([]byte(msg.Payload), &gas_price)

			p := influxdb2.NewPointWithMeasurement("eth_gas_price").
				AddField("rapid", gas_price.Rapid).
				AddField("fast", gas_price.Fast).
				AddField("standard", gas_price.Standard).
				AddField("slow", gas_price.Slow).
				SetTime(utils.FromUnixMilli(gas_price.Timestamp))

			writeAPI.WritePoint(p)
		}
	case config.REDIS_TOPIC_ETH_BLOCK_HEADER:
		{
			block_header := pojo.BlockHeader{}
			json.Unmarshal([]byte(msg.Payload), &block_header)

			p := influxdb2.NewPointWithMeasurement("eth_block_header").
				AddField("number", block_header.Number).
				AddField("miner", block_header.Miner).
				AddField("gasLimit", block_header.GasLimit).
				AddField("gasUsed", block_header.GasUsed).
				AddField("reward", block_header.Reward).
				AddField("reward_usd", block_header.RewardUSD).
				SetTime(time.Unix(block_header.Timestamp, 0))

			writeAPI.WritePoint(p)
		}
	case config.REDIS_TOPIC_CANDLESTICK_EXT:
		{
			candlestick := make(map[string]interface{})
			err := json.Unmarshal([]byte(msg.Payload), &candlestick)
			if err == nil {
				pair := candlestick["pair"].(string)
				arr := strings.Split(pair, "/")

				tags := map[string]string{
					"exchange":    candlestick["exchange"].(string),
					"market_type": candlestick["market_type"].(string),
					"symbol":      candlestick["symbol"].(string),
					"pair":        candlestick["pair"].(string),
					"base":        arr[0],
					"quote":       arr[1],
					"bar_size":    strconv.Itoa(int(candlestick["bar_size"].(float64))),
				}
				delete(candlestick, "exchange")
				delete(candlestick, "market_type")
				delete(candlestick, "symbol")
				delete(candlestick, "pair")
				delete(candlestick, "bar_size")

				p := influxdb2.NewPoint("candlestick_ext",
					tags,
					candlestick,
					utils.FromUnixMilli(int64(candlestick["timestamp"].(float64))),
				)

				writeAPI.WritePoint(p)
			}
		}
	case config.REDIS_TOPIC_CMC_GLOBAL_METRICS:
		{
			global_metrics := make(map[string]interface{})
			err := json.Unmarshal([]byte(msg.Payload), &global_metrics)

			if err == nil {
				tm, _ := time.Parse(time.RFC3339, global_metrics["last_updated"].(string))
				p := influxdb2.NewPoint("cmc_global_metrics",
					map[string]string{},
					global_metrics,
					tm,
				)
				writeAPI.WritePoint(p)
			}

		}
	default:
		log.Fatalf("Unknown channel %s", msg.Channel)
	}
}
