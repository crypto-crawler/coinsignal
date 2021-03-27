package main

import (
	"context"
	"encoding/json"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/buger/jsonparser"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pojo"
	"github.com/soulmachine/coinsignal/pubsub"
)

// Get spot currency prices, mainly for fiat currencies
func fetchFtxMarkets() map[string]float64 {
	url := "https://ftx.com/api/markets"
	client := &http.Client{Timeout: 10 * time.Second}
	req, _ := http.NewRequest("GET", url, nil)

	resp, err := client.Do(req)
	if err != nil {
		panic(err.Error())
	}
	defer resp.Body.Close()

	body, _ := ioutil.ReadAll(resp.Body)
	result, _, _, _ := jsonparser.Get(body, "result")

	var mapping = make(map[string]float64)

	jsonparser.ArrayEach(result, func(element []byte, dataType jsonparser.ValueType, offset int, err error) {
		market_type, _, _, _ := jsonparser.Get(element, "type")
		if string(market_type) == "spot" {
			name, _, _, _ := jsonparser.Get(element, "name")
			arr := strings.Split(string(name), "/")
			base := arr[0]
			quote := arr[1]
			if quote == "USD" || quote == "USDT" {
				price, _, _, _ := jsonparser.Get(element, "price")
				price_float, _ := strconv.ParseFloat(string(price), 64)
				mapping[base] = price_float
			}
		}
	})
	return mapping
}

func main() {

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}
	publisher := pubsub.NewPublisher(context.Background(), redis_url)

	for {
		mapping := fetchFtxMarkets()
		for currency, price := range mapping {
			currency_price := pojo.CurrencyPrice{
				Currency: currency,
				Price:    price,
			}
			json_bytes, _ := json.Marshal(currency_price)

			publisher.Publish(config.REDIS_TOPIC_CURRENCY_PRICE_CHANNEL, string(json_bytes))
		}

		time.Sleep(5 * time.Second)
	}
}
