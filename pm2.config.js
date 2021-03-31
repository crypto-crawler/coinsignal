const apps = [];

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
  name: "data_shipper",
  script: "data_shipper",
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
