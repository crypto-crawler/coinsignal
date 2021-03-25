FROM rust:latest AS rust_builder

RUN mkdir /project
WORKDIR /project

COPY ./rust/ ./
RUN cargo build --release

FROM golang:buster AS go_builder

RUN mkdir /project
WORKDIR /project
COPY ./go/ ./
RUN go build -o cmc_global_metrics cmd/cmc_global_metrics/main.go \
 && go build -o crawler_block_header cmd/crawler_block_header/main.go \
 && go build -o crawler_eth_price cmd/crawler_eth_price/main.go \
 && go build -o crawler_gas_price cmd/crawler_gas_price/main.go


FROM node:buster-slim

COPY --from=rust_builder /project/target/release/trade_crawler /usr/local/bin/
COPY --from=rust_builder /project/target/release/candlestick_builder /usr/local/bin/

COPY --from=go_builder /project/cmc_global_metrics /usr/local/bin/
COPY --from=go_builder /project/crawler_block_header /usr/local/bin/
COPY --from=go_builder /project/crawler_eth_price /usr/local/bin/
COPY --from=go_builder /project/crawler_gas_price /usr/local/bin/

RUN apt-get -qy update && apt-get -qy --no-install-recommends install \
    ca-certificates curl \
 && npm install pm2 -g --production \
 && apt-get -qy autoremove && apt-get clean && rm -rf /var/lib/apt/lists/* && rm -rf /tmp/*

COPY ./pm2.config.js /root/pm2.config.js

ENV RUST_LOG "warn"
ENV RUST_BACKTRACE 1

WORKDIR /root

CMD [ "pm2-runtime", "start", "pm2.config.js" ]
