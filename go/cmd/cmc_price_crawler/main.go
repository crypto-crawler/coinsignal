package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"

	"github.com/buger/jsonparser"
	"github.com/gorilla/websocket"
	"github.com/soulmachine/coinsignal/config"
	"github.com/soulmachine/coinsignal/pojo"
	"github.com/soulmachine/coinsignal/pubsub"
	"github.com/soulmachine/coinsignal/utils"
)

// CoinMarketCap top 100 cryptocurrencies
var currencyMap = map[int64]string{
	1:    "BTC",
	2:    "LTC",
	52:   "XRP",
	74:   "DOGE",
	109:  "DGB",
	131:  "DASH",
	328:  "XMR",
	512:  "XLM",
	825:  "USDT",
	873:  "XEM",
	1027: "ETH",
	1042: "SC",
	1168: "DCR",
	1274: "WAVES",
	1321: "ETC",
	1376: "NEO",
	1437: "ZEC",
	1518: "MKR",
	1684: "QTUM",
	1697: "BAT",
	1720: "MIOTA",
	1727: "BNT",
	1765: "EOS",
	1808: "OMG",
	1817: "VGX",
	1831: "BCH",
	1839: "BNB",
	1886: "DENT",
	1896: "ZRX",
	1958: "TRX",
	1966: "MANA",
	1975: "LINK",
	2010: "ADA",
	2011: "XTZ",
	2099: "ICX",
	2130: "ENJ",
	2135: "REV",
	2280: "FIL",
	2405: "IOST",
	2416: "THETA",
	2469: "ZIL",
	2499: "CHSB",
	2502: "HT",
	2539: "REN",
	2566: "ONT",
	2577: "RVN",
	2586: "SNX",
	2603: "NPXS",
	2682: "HOT",
	2694: "NEXO",
	2700: "CEL",
	3077: "VET",
	3330: "PAX",
	3408: "USDC",
	3513: "FTM",
	3602: "BSV",
	3635: "CRO",
	3673: "BTMX",
	3717: "WBTC",
	3718: "BTT",
	3783: "ANKR",
	3794: "ATOM",
	3822: "TFUEL",
	3890: "MATIC",
	3897: "OKB",
	3945: "ONE",
	3957: "LEO",
	3964: "RSR",
	4023: "BTCB",
	4030: "ALGO",
	4066: "CHZ",
	4157: "RUNE",
	4172: "LUNA",
	4195: "FTT",
	4256: "KLAY",
	4558: "FLOW",
	4642: "HBAR",
	4687: "BUSD",
	4779: "HUSD",
	4847: "STX",
	4943: "DAI",
	4948: "CKB",
	5034: "KSM",
	5426: "SOL",
	5617: "UMA",
	5632: "AR",
	5692: "COMP",
	5777: "RENBTC",
	5805: "AVAX",
	5864: "YFI",
	6535: "NEAR",
	6538: "CRV",
	6636: "DOT",
	6719: "GRT",
	6758: "SUSHI",
	6892: "EGLD",
	7083: "UNI",
	7129: "UST",
	7186: "CAKE",
	7278: "AAVE",
}

func main() {
	ctx := context.Background()

	redis_url := os.Getenv("REDIS_URL")
	if len(redis_url) == 0 {
		log.Fatal("The REDIS_URL environment variable is empty")
	}
	utils.WaitRedis(ctx, redis_url)
	publisher := pubsub.NewPublisher(ctx, redis_url)

	client, _, err := websocket.DefaultDialer.Dial("wss://stream.coinmarketcap.com/price/latest", nil)
	if err != nil {
		log.Fatal(err)
	}

	currencyIds := make([]int64, 0, len(currencyMap))
	for id := range currencyMap {
		currencyIds = append(currencyIds, id)
	}
	command := fmt.Sprintf("{\"method\":\"subscribe\",\"id\":\"price\",\"data\":{\"cryptoIds\":%s,\"index\":null}}", strings.Join(strings.Split(fmt.Sprint(currencyIds), " "), ","))
	// command := "{\"method\":\"subscribe\",\"id\":\"price\",\"data\":{\"cryptoIds\":[1],\"index\":null}}"

	err = client.WriteMessage(websocket.TextMessage, []byte(command))
	if err != nil {
		log.Fatalln("Subscription failed: ", err)
	}

	for {
		_, json_bytes, _ := client.ReadMessage()
		idStr, _, _, _ := jsonparser.Get(json_bytes, "d", "cr", "id")
		priceStr, _, _, _ := jsonparser.Get(json_bytes, "d", "cr", "p")

		id, _ := strconv.ParseInt(string(idStr), 0, 64)
		currency, ok := currencyMap[id]
		if !ok {
			// log.Println("Failed to find symbol for id ", id)
			continue
		}
		price, _ := strconv.ParseFloat(string(priceStr), 64)

		currency_price := &pojo.CurrencyPrice{
			Currency: currency,
			Price:    price,
		}
		json_bytes, _ = json.Marshal(currency_price)
		publisher.Publish(config.REDIS_TOPIC_CURRENCY_PRICE_CHANNEL, string(json_bytes))
	}

	// publisher.Close()
}
