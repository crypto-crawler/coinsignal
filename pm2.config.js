const apps = [
  {
    name: "msg_parser",
    script: "msg_parser",
    exec_interpreter: "none",
    exec_mode: "fork",
    instances: 1,
    restart_delay: 5000, // 5 seconds
  },
  {
    name: "candlestick_builder",
    script: "candlestick_builder",
    exec_interpreter: "none",
    exec_mode: "fork",
    instances: 1,
    restart_delay: 5000, // 5 seconds
  },
  {
    name: "price_updater",
    script: "price_updater",
    exec_interpreter: "none",
    exec_mode: "fork",
    instances: 1,
    restart_delay: 5000, // 5 seconds
  },
  {
    name: "data_shipper",
    script: "data_shipper",
    exec_interpreter: "none",
    exec_mode: "fork",
    instances: 1,
    restart_delay: 5000, // 5 seconds
  },
];

module.exports = {
  apps,
};
