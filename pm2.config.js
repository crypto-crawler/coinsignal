// see src/market_type.rs in crypto-markets
const market_types = {
  binance: [
    "spot",
    "linear_future",
    "inverse_future",
    "linear_swap",
    "inverse_swap",
  ],
  bitfinex: ["spot", "linear_swap"],
  bitget: ["inverse_swap", "linear_swap"],
  bithumb: ["spot"],
  bitmex: [
    "inverse_swap",
    "quanto_swap",
    "linear_future",
    "inverse_future",
    "quanto_future",
  ],
  bitstamp: ["spot"],
  bitz: ["spot"],
  bybit: ["inverse_future", "inverse_swap", "linear_swap"],
  coinbase_pro: ["spot"],
  deribit: ["inverse_future", "option"], // inverse_swap is included in inverse_future
  ftx: ["spot", "linear_swap", "linear_future", "move", "bvol"],
  gate: ["spot", "linear_future", "linear_swap", "inverse_swap"],
  huobi: ["spot", "inverse_future", "linear_swap", "inverse_swap", "option"],
  kraken: ["spot"],
  kucoin: ["spot", "inverse_future", "linear_swap", "inverse_swap"],
  mxc: ["spot", "linear_swap", "inverse_swap"],
  okex: [
    "spot",
    "linear_future",
    "inverse_future",
    "linear_swap",
    "inverse_swap",
    "option",
  ],
  zbg: ["spot", "inverse_swap", "linear_swap"],
};

const apps = [];

Object.keys(market_types).forEach((exchange) => {
  market_types[exchange].forEach((market_ype) => {
    const app = {
      name: `trade-crawler-${exchange}-${market_ype}`,
      script: "trade_crawler",
      args: `${exchange} ${market_ype}`,
      exec_interpreter: "none",
      exec_mode: "fork",
      instances: 1,
      restart_delay: 5000, // 5 seconds
    };

    apps.push(app);
  });
});

apps.push({
  name: "candlestick_builder",
  script: "candlestick_builder",
  exec_interpreter: "none",
  exec_mode: "fork",
  instances: 1,
  restart_delay: 5000, // 5 seconds
});

apps.push({
  name: "price_updater",
  script: "price_updater",
  exec_interpreter: "none",
  exec_mode: "fork",
  instances: 1,
  restart_delay: 5000, // 5 seconds
});

apps.push({
  name: "cmc_global_metrics",
  script: "cmc_global_metrics",
  exec_interpreter: "none",
  exec_mode: "fork",
  cron_restart: "*/5 * * * *",
  autorestart: false,
});

apps.push({
  name: "cmc_price_crawler",
  script: "cmc_price_crawler",
  exec_interpreter: "none",
  exec_mode: "fork",
  instances: 1,
  restart_delay: 5000, // 5 seconds
});

apps.push({
  name: "crawler_block_header",
  script: "crawler_block_header",
  exec_interpreter: "none",
  exec_mode: "fork",
  instances: 1,
  restart_delay: 5000, // 5 seconds
});

apps.push({
  name: "crawler_gas_price",
  script: "crawler_gas_price",
  exec_interpreter: "none",
  exec_mode: "fork",
  instances: 1,
  restart_delay: 5000, // 5 seconds
});

apps.push({
  name: "crawler_mark_price",
  script: "crawler_mark_price",
  exec_interpreter: "none",
  exec_mode: "fork",
  instances: 1,
  restart_delay: 5000, // 5 seconds
});

apps.push({
  name: "ftx_spot_price",
  script: "ftx_spot_price",
  exec_interpreter: "none",
  exec_mode: "fork",
  instances: 1,
  restart_delay: 5000, // 5 seconds
});

module.exports = {
  apps,
};
