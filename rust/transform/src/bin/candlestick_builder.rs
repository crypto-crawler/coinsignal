use crypto_market_type::MarketType;
use crypto_msg_parser::{TradeMsg, TradeSide};
use lazy_static::lazy_static;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use transform::constants::*;
use utils::{pubsub::Publisher, wait_redis, PriceCache};

lazy_static! {
    // see https://www.stablecoinswar.com/
    static ref STABLE_COINS: HashSet<&'static str> = vec![
        "USD", "USDT", "USDC", "BUSD", "DAI", "PAX", "HUSD", "TUSD", "GUSD", "USDK"
        ].into_iter().collect();

    static ref REDIS_URL: &'static str = if std::env::var("REDIS_URL").is_err() {
            info!(
                "The REDIS_URL environment variable is empty, using redis://localhost:6379 by default"
            );
            "redis://localhost:6379"
        } else {
            let mut url = std::env::var("REDIS_URL").unwrap();
            if !url.starts_with("redis://") {
                url = format!("redis://{}", url);
            }
            Box::leak(url.into_boxed_str())
        };

    static ref PRICE_CACHE: PriceCache = PriceCache::new(*REDIS_URL);
}

fn extract_quote(pair: &str) -> &str {
    if pair.find('/').is_none() {
        warn!("{}", pair);
    }
    let slash_pos = pair.find('/').unwrap();
    &pair[slash_pos + 1..]
}

fn is_good(quote: &str) -> bool {
    STABLE_COINS.contains(quote) || PRICE_CACHE.get_price(quote).is_some()
}

#[derive(Serialize, Deserialize)]
pub struct Candlestick {
    exchange: String,
    market_type: MarketType,
    symbol: String,
    pair: String,
    bar_size: i64,        // in millisecond
    pub timestamp: i64,   // bar end time, in millisecond
    timestamp_start: i64, // timestamp of the fist trade
    timestamp_end: i64,   // timestamp of the last trade

    open: f64,
    high: f64,
    low: f64,
    close: f64,

    volume: f64,      // base volume
    volume_sell: f64, // base volume at sell side
    volume_buy: f64,  // base volume at buy side

    volume_quote: f64,      // quote volume
    volume_quote_sell: f64, // quote volume at sell side
    volume_quote_buy: f64,  // quote volume at buy side

    volume_usd: f64,      // volume converted to USD
    volume_usd_sell: f64, // volume_usd at sell side
    volume_usd_buy: f64,  // volume_usd at buy side

    volume_btc: f64,      // volume converted to BTC
    volume_btc_sell: f64, // volume_btc at sell side
    volume_btc_buy: f64,  // volume_btc at buy side

    vwap: f64,     // volume weighted average price in quote currency
    vwap_usd: f64, // volume weighted average price in USD
    vwap_btc: f64, // volume weighted average price in BTC

    count: i64,      // number of trades
    count_sell: i64, // number of sell trades
    count_buy: i64,  // number of buy trades

    #[serde(skip_serializing)]
    dedup: HashSet<u64>,
}

impl Candlestick {
    pub fn new(
        exchange: String,
        market_type: MarketType,
        symbol: String,
        pair: String,
        bar_size: i64,  // in second, BTC, ETH, USD, etc.
        timestamp: i64, // bar end time
    ) -> Self {
        Candlestick {
            exchange: exchange,
            market_type,
            symbol,
            pair,
            bar_size,
            timestamp,
            timestamp_start: 0,
            timestamp_end: 0,

            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,

            volume: 0.0,
            volume_sell: 0.0,
            volume_buy: 0.0,
            volume_quote: 0.0,
            volume_quote_sell: 0.0,
            volume_quote_buy: 0.0,
            volume_usd: 0.0,
            volume_usd_sell: 0.0,
            volume_usd_buy: 0.0,
            volume_btc: 0.0,
            volume_btc_sell: 0.0,
            volume_btc_buy: 0.0,

            vwap: 0.0,
            vwap_usd: 0.0,
            vwap_btc: 0.0,

            count: 0,
            count_sell: 0,
            count_buy: 0,

            dedup: HashSet::new(),
        }
    }

    pub fn append(&mut self, trade: &TradeMsg) -> bool {
        if trade.exchange != self.exchange {
            warn!(
                "The trade's exchange {} is not equal to candlestick exchange {}",
                trade.exchange, self.exchange
            );
            return false;
        }
        if trade.market_type != self.market_type {
            warn!(
                "The trade's market_type {} is not equal to candlestick market_type {}",
                trade.market_type, self.market_type
            );
            return false;
        }
        if trade.symbol != self.symbol {
            warn!(
                "The trade's symbol {} is not equal to candlestick symbol {}",
                trade.symbol, self.symbol
            );
            return false;
        }
        if trade.pair != self.pair {
            warn!(
                "The trade's pair {} is not equal to candlestick pair {}",
                trade.pair, self.pair
            );
            return false;
        }
        if trade.timestamp >= self.timestamp {
            warn!(
                "The trade's timestamp {} is greater or equal than candlestick end timestamp {}",
                trade.timestamp, self.timestamp
            );
            return false;
        } else if trade.timestamp < (self.timestamp - self.bar_size) {
            warn!(
                "The trade's timestamp {} is less than candlestick's begin timestamp {}",
                trade.timestamp,
                self.timestamp - self.bar_size
            );
            return false;
        }

        if self.dedup.contains(&Self::calc_trade_hash(trade)) {
            warn!(
                "Found duplicated trade {} ",
                serde_json::to_string(trade).unwrap()
            );
            return false;
        }

        let quote = extract_quote(&trade.pair);
        if !is_good(quote) {
            warn!(
                "The trade's quote symbol {} is neither stable coin nor in PRICE_CACHE",
                quote
            );
            return false;
        }
        let quote_price = if STABLE_COINS.contains(quote) {
            1.0
        } else {
            PRICE_CACHE.get_price(quote).unwrap()
        };

        if PRICE_CACHE.get_price("BTC").is_none() {
            warn!("PRICE_CACHE does NOT have BTC");
            return false;
        }
        let btc_price = PRICE_CACHE.get_price("BTC").unwrap();

        if self.count == 0 {
            self.timestamp_start = trade.timestamp;
            self.timestamp_end = trade.timestamp;

            self.open = trade.price;
            self.high = trade.price;
            self.low = trade.price;
            self.close = trade.price;
        } else {
            if self.timestamp_start > trade.timestamp {
                self.timestamp_start = trade.timestamp;
                self.open = trade.price;
            }

            if self.timestamp_end < trade.timestamp {
                self.timestamp_end = trade.timestamp;
                self.close = trade.price;
            }

            if self.high < trade.price {
                self.high = trade.price;
            }
            if self.low > trade.price {
                self.low = trade.price;
            }
        }

        let volume_delta = trade.quantity;
        let volume_quote_delta = trade.volume;
        let volume_usd_delta = trade.volume * quote_price;
        let volume_btc_delta = trade.volume * quote_price / btc_price;

        self.volume += volume_delta;
        self.volume_quote += volume_quote_delta;
        self.volume_usd += volume_usd_delta;
        self.volume_btc += volume_btc_delta;

        self.count += 1;
        if trade.side == TradeSide::Sell {
            self.volume_sell += volume_delta;
            self.volume_quote_sell += volume_quote_delta;
            self.volume_usd_sell += volume_usd_delta;
            self.volume_btc_sell += volume_btc_delta;

            self.count_sell += 1;
        } else {
            self.volume_buy += volume_delta;
            self.volume_quote_buy += volume_quote_delta;
            self.volume_usd_buy += volume_usd_delta;
            self.volume_btc_buy += volume_btc_delta;
            self.count_buy += 1;
        }

        true
    }

    pub fn finalize(&mut self) {
        self.vwap = self.volume_quote / self.volume;
        self.vwap_usd = self.volume_usd / self.volume;
        self.vwap_btc = self.volume_btc / self.volume;
    }

    fn calc_trade_hash(trade: &TradeMsg) -> u64 {
        let mut s = DefaultHasher::new();

        trade.timestamp.hash(&mut s);
        trade.trade_id.hash(&mut s);
        trade.price.to_string().hash(&mut s);
        trade.quantity.to_string().hash(&mut s);
        trade.volume.to_string().hash(&mut s);
        trade.side.to_string().hash(&mut s);

        s.finish()
    }
}

const INTERVAL: i64 = 300000; // 5 minutes in milliseconds

// Merge trades into 1-minute klines
fn main() {
    env_logger::init();
    PRICE_CACHE.wait_until_ready();

    let redis_url = if std::env::var("REDIS_URL").is_err() {
        info!(
            "The REDIS_URL environment variable is empty, using redis://localhost:6379 by default"
        );
        "redis://localhost:6379"
    } else {
        let mut url = std::env::var("REDIS_URL").unwrap();
        if !url.starts_with("redis://") {
            url = format!("redis://{}", url);
        }
        Box::leak(url.into_boxed_str())
    };
    wait_redis(redis_url);

    let mut publisher = Publisher::new(redis_url);

    // subscriber
    let mut connection = {
        let client = redis::Client::open(redis_url).unwrap();
        client.get_connection().unwrap()
    };
    let mut pubsub = connection.as_pubsub();
    pubsub.subscribe(REDIS_TOPIC_TRADE).unwrap();

    let mut candlesticks: HashMap<String, Candlestick> = HashMap::new();

    let mut current_bar_time = {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        now / INTERVAL * INTERVAL + INTERVAL
    };

    loop {
        let trade_msg = {
            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();
            serde_json::from_str::<TradeMsg>(&payload).unwrap()
        };
        if !is_good(extract_quote(&trade_msg.pair)) {
            // warn!(
            //     "{}, {}, {}",
            //     trade_msg.exchange, trade_msg.market_type, trade_msg.pair
            // );
            continue;
        }

        let msg_bar_time = (trade_msg.timestamp / INTERVAL) * INTERVAL + INTERVAL;
        let key = format!(
            "{}-{}-{}-{}-{}",
            trade_msg.exchange,
            trade_msg.market_type,
            trade_msg.pair,
            trade_msg.symbol,
            msg_bar_time
        );
        if !candlesticks.contains_key(&key) {
            candlesticks.insert(
                key.clone(),
                Candlestick::new(
                    trade_msg.exchange.clone(),
                    trade_msg.market_type,
                    trade_msg.symbol.clone(),
                    trade_msg.pair.clone(),
                    INTERVAL,
                    msg_bar_time,
                ),
            );
        }
        let candlestick = candlesticks.get_mut(&key).unwrap();
        candlestick.append(&trade_msg);

        let bar_time = {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            now / INTERVAL * INTERVAL + INTERVAL
        };
        if bar_time >= current_bar_time {
            // output
            let keys: Vec<String> = candlesticks.keys().cloned().collect();
            for key in keys.iter() {
                let mut candlestick = candlesticks.remove(key).unwrap();
                if candlestick.timestamp < current_bar_time {
                    candlestick.finalize();
                    publisher.publish::<Candlestick>(REDIS_TOPIC_CANDLESTICK_EXT, &candlestick);
                }
            }

            current_bar_time = bar_time;
        }
    }
}
