package config

// carbonbot Redis channels
const REDIS_TOPIC_PREFIX = "carbonbot:misc:"
const REDIS_TOPIC_ETH_BLOCK_HEADER = REDIS_TOPIC_PREFIX + "eth_block_header"
const REDIS_TOPIC_CMC_GLOBAL_METRICS = REDIS_TOPIC_PREFIX + "cmc_global_metrics"
const REDIS_TOPIC_ETH_GAS_PRICE = REDIS_TOPIC_PREFIX + "eth_gas_price"

// coinsignal Redis channels
const REDIS_COINSIGNAL_TOPIC_PREFIX = "coinsignal:"
const REDIS_TOPIC_CANDLESTICK_EXT = REDIS_COINSIGNAL_TOPIC_PREFIX + "candlestick_ext"
const REDIS_TOPIC_FUNDING_RATE_PARSED = REDIS_COINSIGNAL_TOPIC_PREFIX + "funding_rate"
