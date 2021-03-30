package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"strconv"
	"time"

	"github.com/buger/jsonparser"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pubsub"
	"github.com/soulmachine/coinsignal/utils"
)

// return ETH number
func fetchBlockReward(blockNumber int64) float64 {
	etherscan_api_key := os.Getenv("ETHERSCAN_API_KEY")
	if len(etherscan_api_key) == 0 {
		log.Fatal("The ETHERSCAN_API_KEY environment variable is empty")
	}
	url := fmt.Sprintf("https://api.etherscan.io/api?module=block&action=getblockreward&blockno=%d&apikey=%s", blockNumber, etherscan_api_key)

	for i := 0; i < 3; i++ {
		time.Sleep(5 * time.Second)
		resp, err := http.Get(url)
		if err != nil {
			log.Fatalln(err)
		}
		defer resp.Body.Close()
		body, _ := ioutil.ReadAll(resp.Body)

		message, _, _, _ := jsonparser.Get(body, "message")
		blockRewardStr, _, _, _ := jsonparser.Get(body, "result", "blockReward")
		if string(message) == "OK" {
			blockReward, _ := strconv.ParseUint(string(blockRewardStr), 10, 64)
			return float64(blockReward) / 1000000000000000000
		}
	}

	return 4.54104 // default value, see https://bitinfocharts.com/ethereum/
}

func main() {
	ctx := context.Background()

	full_node_url := os.Getenv("FULL_NODE_URL")
	if len(full_node_url) == 0 {
		log.Fatal("The FULL_NODE_URL environment variable is empty")
	}

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}
	utils.WaitRedis(ctx, redis_url)

	priceCache := utils.NewPriceCache(ctx, redis_url)
	priceCache.WaitUntilReady()

	client, err := ethclient.Dial(full_node_url)
	if err != nil {
		log.Fatal(err)
	}

	headers := make(chan *types.Header)
	sub, err := client.SubscribeNewHead(context.Background(), headers)
	if err != nil {
		log.Fatal(err)
	}

	publisher := pubsub.NewPublisher(ctx, redis_url)
	for {
		select {
		case err := <-sub.Err():
			log.Fatal(err)
		case header := <-headers:
			json_bytes, _ := json.Marshal(header)

			blockNumberBytes, _, _, _ := jsonparser.Get(json_bytes, "number")
			blockNumber, _ := strconv.ParseInt(string(blockNumberBytes), 0, 64)
			blockReward := fetchBlockReward(blockNumber)

			ethPrice := priceCache.GetPrice("ETH")

			blockRewardUSD := blockReward * ethPrice

			json_bytes, _ = jsonparser.Set(json_bytes, []byte(strconv.FormatFloat(blockReward, 'f', -1, 64)), "reward")
			json_bytes, _ = jsonparser.Set(json_bytes, []byte(strconv.FormatFloat(blockRewardUSD, 'f', -1, 64)), "reward_usd")

			gasLimitBytes, _, _, _ := jsonparser.Get(json_bytes, "gasLimit")
			gasLimit, _ := strconv.ParseInt(string(gasLimitBytes), 0, 64)
			gasUsedBytes, _, _, _ := jsonparser.Get(json_bytes, "gasUsed")
			gasUsed, _ := strconv.ParseInt(string(gasUsedBytes), 0, 64)
			timestampBytes, _, _, _ := jsonparser.Get(json_bytes, "timestamp")
			timestamp, _ := strconv.ParseInt(string(timestampBytes), 0, 64)

			json_bytes, _ = jsonparser.Set(json_bytes, []byte(strconv.FormatInt(blockNumber, 10)), "number")
			json_bytes, _ = jsonparser.Set(json_bytes, []byte(strconv.FormatInt(gasLimit, 10)), "gasLimit")
			json_bytes, _ = jsonparser.Set(json_bytes, []byte(strconv.FormatInt(gasUsed, 10)), "gasUsed")
			json_bytes, _ = jsonparser.Set(json_bytes, []byte(strconv.FormatInt(timestamp, 10)), "timestamp")

			if ethPrice > 0.0 {
				publisher.Publish(config.REDIS_TOPIC_ETH_BLOCK_HEADER, string(json_bytes))
			}
		}
	}

	// publisher.Close()
	// priceCache.Close()
}
