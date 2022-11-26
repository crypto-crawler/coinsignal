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

![Architecture](./architecture.svg)

- I tried to use Kafka as the message queue, but it's too heavy, so I used Redis instead.

## How to deploy in production

### 1. Redis and InfluxDB

Make sure Redis and InfluxDB are running.

### 2. Crawlers

`carbonbot-trade`:

```bash
docker run -d --name carbonbot-trade --restart always \
  -v $LOCAL_TMP_DIR:/carbonbot_data \
  -v $DEST_DIR:/dest_dir \
  -e DEST_DIR=/dest_dir \
  -e REDIS_URL="redis://172.17.0.1:6379" \
  -u "$(id -u):$(id -g)" ghcr.io/crypto-crawler/carbonbot:latest pm2-runtime start pm2.trade.config.js
```

`carbonbot-misc`:

```bash
docker run -d --name carbonbot-misc --restart always \
  -v $LOCAL_TMP_DIR:/carbonbot_data \
  -v $DEST_DIR:/dest_dir \
  -e DEST_DIR=/dest_dir \
  -e REDIS_URL="redis://172.17.0.1:6379" \
  -e FULL_NODE_URL="wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID" \
  -e ETHERSCAN_API_KEY=YOUR_API_KEY \
  -e CMC_API_KEY=YOUR_API_KEY \
  -u "$(id -u):$(id -g)" soulmachine/carbonbot:misc
```

### 3. Backend

```bash
docker run -d --name coinsignal-backend --restart always \
  -e INFLUXDB_URL=http://172.17.0.1:8086 \
  -e INFLUXDB_ORG=crypto-crawler \
  -e INFLUXDB_BUCKET=coinsignal \
  -e INFLUXDB_TOKEN=YOUR_TOKEN \
  -e REDIS_URL="redis://172.17.0.1:6379" \
  ghcr.io/crypto-crawler/coinsignal:backend
```

### 4. Frontend

```bash
docker run -d --name coinsignal-frontend --restart always \
  -e INFLUXDB_URL=http://172.17.0.1:8086 \
  -e INFLUXDB_ORG=crypto-crawler \
  -e INFLUXDB_BUCKET=coinsignal \
  -e INFLUXDB_TOKEN=YOUR_TOKEN \
  -e GF_SERVER_DOMAIN=coinsignal.org \
  -e GF_AUTH_ANONYMOUS_ENABLED=true \
  -e GF_AUTH_BASIC_ENABLED=false \
  -e GF_AUTH_DISABLE_LOGIN_FORM=true \
  -p 80:3000 \
  ghcr.io/crypto-crawler/coinsignal:frontend
```

The differences between dev and prod are:

- `-e GF_SECURITY_ADMIN_USER=admin -e GF_SECURITY_ADMIN_PASSWORD=YOUR_PASSWORD` vs. `-e GF_SERVER_DOMAIN=coinsignal.org -e GF_AUTH_ANONYMOUS_ENABLED=true -e GF_AUTH_BASIC_ENABLED=false -e GF_AUTH_DISABLE_LOGIN_FORM=true`
- `-p 3000:3000` vs. `-p 80:3000`

You can run two frontend containers in parallel, one for development and one for production.

## 5. Build Images

```bash
docker build -t ghcr.io/crypto-crawler/coinsignal:backend . -f Dockerfile.backend
docker push ghcr.io/crypto-crawler/coinsignal:backend

docker build -t ghcr.io/crypto-crawler/coinsignal:frontend . -f Dockerfile.frontend
docker push ghcr.io/crypto-crawler/coinsignal:frontend
```
