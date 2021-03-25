package config

const REDIS_TOPIC_PREFIX = "coinsignal:"
const REDIS_TOPIC_ETH_BLOCK_HEADER = REDIS_TOPIC_PREFIX + "eth_block_header"
const REDIS_TOPIC_CMC_GLOBAL_METRICS = REDIS_TOPIC_PREFIX + "cmc_global_metrics"
const REDIS_TOPIC_ETH_PRICE = REDIS_TOPIC_PREFIX + "eth_price"
const REDIS_TOPIC_ETH_GAS_PRICE = REDIS_TOPIC_PREFIX + "eth_gas_price"
