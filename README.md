# coinsignal

Calculate all kinds of indicators to assist cryptocurrency trading.

Website: <http://coinsignal.org>

If you want to run this project locally, please read on.

## Quickstart

First, apply some API keys and save them to a file named `.env`:

```ini
ETHERSCAN_API_KEY="your etherscan.io API key"
FULL_NODE_URL="wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID"
CMC_API_KEY="your coinmarketcap.com API key"
```

Second, run coinsignal,

```bash
docker-compose --env-file .envrc up
```

Open <http://localhost:3000> in browser and login with `admin` and `passw0rd`, enjoy!

Additionally, you can open influxdb at <http://localhost:8086>

## Build

```bash
docker build -t soulmachine/coinsignal:frontend . -f Dockerfile.frontend
docker build -t soulmachine/coinsignal:backend . -f Dockerfile.backend
docker push soulmachine/coinsignal:frontend && docker push soulmachine/coinsignal:backend
```

## Architecture

![Architecture](./architecture.png)

- I tried to use Kafka as the message queue, but it's too heavy, so I used Redis instead.

## How to deploy in production

### 1. Redis and InfluxDB

Make sure Redis and InfluxDB are running.

### 2. Crawlers

`carbonbot-trade`:

```bash
docker run -d --name carbonbot-trade --restart always -e REDIS_URL="redis://:7BUvEvH@192.168.5.250:6379" -u "$(id -u):$(id -g)" soulmachine/carbonbot pm2-runtime start pm2.trade.config.js
```

`carbonbot-misc`:

```bash
docker run -d --name carbonbot-misc --restart always -e REDIS_URL="redis://:password@ip:6379" -e FULL_NODE_URL="wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID" -e ETHERSCAN_API_KEY="YOUR_API_KEY" -e CMC_API_KEY="YOUR_API_KEY" -u "$(id -u):$(id -g)" soulmachine/carbonbot:misc
```

### 3. Backend

```bash
docker run -d --name coinsignal-backend --restart always -e INFLUXDB_URL=http://ip:8086 -e INFLUXDB_ORG=ORG_NAME -e INFLUXDB_BUCKET=BUCKET_NAME -e INFLUXDB_TOKEN=YOUR_TOKEN -e REDIS_URL="redis://:password@ip:6379" soulmachine/coinsignal:backend
```

### 4. Frontend

```bash
docker run -d --name coinsignal-frontend --restart always -e INFLUXDB_URL=http://ip:8086 -e INFLUXDB_ORG=ORG_NAME -e INFLUXDB_BUCKET=BUCKET_NAME -e INFLUXDB_TOKEN=YOUR_TOKEN -e GF_SECURITY_ADMIN_USER=admin -e GF_SECURITY_ADMIN_PASSWORD=YOUR_PASSWORD -p 80:3000 soulmachine/coinsignal:frontend
```
