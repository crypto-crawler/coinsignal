# coinsignal

Calculate all kinds of indicators to assist cryptocurrency trading.

Website: <http://coinsignal.org>

If you want to run this project locally, please read on.

## Quickstart

First, create a file named `.env` to store your API keys,

```ini
ETHERSCAN_API_KEY="your etherscan.io API key"
FULL_NODE_URL="wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID"
CMC_API_KEY="your coinmarketcap.com API key"
```

Second, run coinsignal,

```bash
docker-compose --env-file .envrc up
```

Third, open <http://localhost:3000> in browser and login with `admin` and `passw0rd`.

Additionally, open Influxdb at <http://localhost:8086>

## Build

```bash
docker build -t soulmachine/coinsignal:frontend . -f Dockerfile.frontend
docker build -t soulmachine/coinsignal:backend . -f Dockerfile.backend
docker push soulmachine/coinsignal:frontend && docker push soulmachine/coinsignal:backend
```

## Architecture

![Architecture](./architecture.png)

- I tried to use Kafka as the message queue, but it's too heavy, so I used Redis instead.
