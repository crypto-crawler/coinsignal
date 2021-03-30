FROM rust:latest AS rust_builder

RUN mkdir /project
WORKDIR /project

COPY ./rust/ ./
RUN cargo build --release

FROM golang:latest AS go_builder

RUN mkdir /project
WORKDIR /project
COPY ./go/ ./
RUN go build -o cmc_global_metrics cmd/cmc_global_metrics/main.go \
 && go build -o cmc_price_crawler cmd/cmc_price_crawler/main.go \
 && go build -o crawler_block_header cmd/crawler_block_header/main.go \
 && go build -o crawler_gas_price cmd/crawler_gas_price/main.go \
 && go build -o crawler_mark_price cmd/crawler_mark_price/main.go \
 && go build -o data_shipper cmd/data_shipper/main.go \
 && go build -o ftx_spot_price cmd/ftx_spot_price/main.go


FROM node:buster-slim

COPY --from=rust_builder /project/target/release/trade_crawler /usr/local/bin/
COPY --from=rust_builder /project/target/release/candlestick_builder /usr/local/bin/
COPY --from=rust_builder /project/target/release/price_updater /usr/local/bin/

COPY --from=go_builder /project/cmc_global_metrics /usr/local/bin/
COPY --from=go_builder /project/cmc_price_crawler /usr/local/bin/
COPY --from=go_builder /project/crawler_block_header /usr/local/bin/
COPY --from=go_builder /project/crawler_gas_price /usr/local/bin/
COPY --from=go_builder /project/crawler_mark_price /usr/local/bin/
COPY --from=go_builder /project/data_shipper /usr/local/bin/
COPY --from=go_builder /project/ftx_spot_price /usr/local/bin/

RUN apt-get -qy update && apt-get -qy --no-install-recommends install \
    ca-certificates curl redis-server \
 && npm install pm2 -g --production \
 && apt-get -qy autoremove && apt-get clean && rm -rf /var/lib/apt/lists/* && rm -rf /tmp/*

COPY ./pm2.config.js /root/pm2.config.js

ENV REDIS_URL localhost:6379
EXPOSE 6379

ENV RUST_LOG "warn"
ENV RUST_BACKTRACE 1

WORKDIR /root

CMD [ "pm2-runtime", "start", "pm2.config.js" ]
