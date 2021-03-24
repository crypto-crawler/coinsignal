FROM rust:latest AS builder

RUN mkdir /projects
WORKDIR /projects

COPY ./rust ./
RUN cargo build --release


FROM node:buster-slim

COPY --from=builder /projects/target/release/trade_crawler /usr/local/bin/
COPY --from=builder /projects/target/release/candlestick_builder /usr/local/bin/

RUN apt-get -qy update && apt-get -qy --no-install-recommends install \
    ca-certificates curl \
 && npm install pm2 -g \
 && apt-get -qy autoremove && apt-get clean && rm -rf /var/lib/apt/lists/* && rm -rf /tmp/*

COPY ./pm2.config.js /root/pm2.config.js

ENV RUST_LOG "warn"
ENV RUST_BACKTRACE 1
