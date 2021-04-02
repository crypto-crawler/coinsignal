package config

const REDIS_TOPIC_PREFIX = "coinsignal:"
const REDIS_TOPIC_ETH_BLOCK_HEADER = REDIS_TOPIC_PREFIX + "eth_block_header"
const REDIS_TOPIC_CMC_GLOBAL_METRICS = REDIS_TOPIC_PREFIX + "cmc_global_metrics"
const REDIS_TOPIC_CURRENCY_PRICE = REDIS_TOPIC_PREFIX + "currency_price"
const REDIS_TOPIC_CURRENCY_PRICE_CHANNEL = REDIS_TOPIC_PREFIX + "currency_price_channel"
const REDIS_TOPIC_ETH_GAS_PRICE = REDIS_TOPIC_PREFIX + "eth_gas_price"
const REDIS_TOPIC_CANDLESTICK_EXT = REDIS_TOPIC_PREFIX + "candlestick_ext"
const REDIS_TOPIC_FUNDING_RATE = "carbonbot:funding_rate"
