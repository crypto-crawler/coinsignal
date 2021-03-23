use crypto_market_type::MarketType;
use crypto_msg_parser::{TradeMsg, TradeSide};
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use transform::constants::*;
use utils::pubsub::Publisher;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct Candlestick {
    exchange: String,
    market_type: MarketType,
    symbol: String,
    pair: String,
    bar_size: i64,        // in second, BTC, ETH, USD, etc.
    timestamp: i64,       // end time
    timestamp_start: i64, // start time

    open: f64,
    high: f64,
    low: f64,
    close: f64,
    mean: f64,
    median: f64,

    volume: f64,            // base volume
    volume_quote: f64,      // quote volume
    volume_sell: f64,       // base volume at sell side
    volume_buy: f64,        // base volume at buy side
    volume_quote_sell: f64, // quote volume at sell side
    volume_quote_buy: f64,  // quote volume at buy side

    vwap: f64, // volume weighted average price

    count: i64,      // number of trades
    count_sell: i64, // number of sell trades
    count_buy: i64,  // number of buy trades
}

const INTERVAL: i64 = 60000; // 1 minutes in milliseconds

struct OHLCVMsg {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    mean: f64,
    median: f64,
}

fn aggregate(nums: &mut Vec<f64>) -> OHLCVMsg {
    assert!(!nums.is_empty());
    let open = nums[0];
    let close = nums[nums.len() - 1];

    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let low = nums[0];
    let high = nums[nums.len() - 1];

    let mean = nums.iter().sum::<f64>() / nums.len() as f64;

    let mid = nums.len() / 2;

    let median = if nums.len() % 2 == 0 {
        (nums[mid] + nums[mid - 1]) / 2.0
    } else {
        nums[mid]
    };

    OHLCVMsg {
        open,
        high,
        low,
        close,
        mean,
        median,
    }
}

fn build_candlestick(bar_time: i64, bar_size: i64, trades: &mut [TradeMsg]) -> Candlestick {
    assert!(!trades.is_empty());
    trades.sort_by(|x, y| {
        if x.timestamp == y.timestamp {
            x.trade_id.cmp(&y.trade_id)
        } else {
            x.timestamp.partial_cmp(&y.timestamp).unwrap()
        }
    });

    let mut prices: Vec<f64> = trades.iter().map(|x| x.price).collect();
    let price_ohlc: OHLCVMsg = aggregate(&mut prices);

    let volume = trades.iter().map(|x| x.quantity).sum();
    let volume_sell = trades
        .iter()
        .filter(|x| x.side == TradeSide::Sell)
        .map(|x| x.quantity)
        .sum();
    let volume_buy = trades
        .iter()
        .filter(|x| x.side == TradeSide::Buy)
        .map(|x| x.quantity)
        .sum();
    let volume_quote = trades.iter().map(|x| x.volume).sum();
    let volume_quote_sell = trades
        .iter()
        .filter(|x| x.side == TradeSide::Sell)
        .map(|x| x.volume)
        .sum();
    let volume_quote_buy = trades
        .iter()
        .filter(|x| x.side == TradeSide::Buy)
        .map(|x| x.volume)
        .sum();

    Candlestick {
        exchange: trades[0].exchange.clone(),
        market_type: trades[0].market_type,
        symbol: trades[0].symbol.clone(),
        pair: trades[0].pair.clone(),
        bar_size,
        timestamp: bar_time,
        timestamp_start: trades[0].timestamp,
        open: price_ohlc.open,
        high: price_ohlc.high,
        low: price_ohlc.low,
        close: price_ohlc.close,
        mean: price_ohlc.mean,
        median: price_ohlc.median,
        volume,
        volume_sell,
        volume_buy,
        volume_quote,
        volume_quote_sell,
        volume_quote_buy,

        vwap: volume_quote / volume,

        count: trades.len() as i64,
        count_sell: trades.iter().filter(|x| x.side == TradeSide::Sell).count() as i64,
        count_buy: trades.iter().filter(|x| x.side == TradeSide::Buy).count() as i64,
    }
}

// Merge trades into 1-minute klines
fn main() {
    env_logger::init();

    let redis_url: &'static str = if std::env::var("REDIS_URL").is_err() {
        warn!(
            "The REDIS_URL environment variable is empty, using redis://localhost:6379 by default"
        );
        "redis://localhost:6379"
    } else {
        Box::leak(std::env::var("REDIS_URL").unwrap().into_boxed_str())
    };
    let mut publisher = Publisher::new(redis_url);

    let mut prev_bar_time_end = -1i64;
    let mut prev_bar_time_begin = -1i64;
    let mut cur_bar_time_end = -1i64;

    let mut cache_prev = HashMap::<String, Vec<TradeMsg>>::new();
    let mut cache = HashMap::<String, Vec<TradeMsg>>::new();

    // subscriber
    let client = redis::Client::open(redis_url).unwrap();
    let mut connection = client.get_connection().unwrap();
    let mut pubsub = connection.as_pubsub();
    pubsub.subscribe(REDIS_TOPIC_TRADE).unwrap();

    loop {
        let msg = pubsub.get_message().unwrap();
        let payload: String = msg.get_payload().unwrap();
        let trade_msg = serde_json::from_str::<TradeMsg>(&payload).unwrap();

        let key = format!(
            "{}-{}-{}-{}",
            trade_msg.exchange, trade_msg.market_type, trade_msg.pair, trade_msg.symbol
        );

        if prev_bar_time_end == -1 {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
            prev_bar_time_end = now / INTERVAL * INTERVAL;
            prev_bar_time_begin = prev_bar_time_end - INTERVAL;
            cur_bar_time_end = prev_bar_time_end + INTERVAL;
        }

        if trade_msg.timestamp < prev_bar_time_begin {
            warn!(
                "Expired msg, prev_bar_time_begin: {}, trade_msg: {}",
                prev_bar_time_begin,
                serde_json::to_string(&trade_msg).unwrap()
            );
        } else if trade_msg.timestamp < prev_bar_time_end {
            if !cache_prev.contains_key(&key) {
                cache_prev.insert(key.clone(), Vec::new());
            }
            cache_prev.get_mut(&key).unwrap().push(trade_msg);
        } else if trade_msg.timestamp < cur_bar_time_end {
            if !cache.contains_key(&key) {
                cache.insert(key.clone(), Vec::new());
            }
            cache.get_mut(&key).unwrap().push(trade_msg);
        } else {
            // build 1-minute TimeBar from cache_prev
            let keys: Vec<String> = cache_prev.keys().cloned().collect();
            for key in keys.iter() {
                let trades = cache_prev.get_mut(key).unwrap();
                let bar = build_candlestick(prev_bar_time_end, INTERVAL, trades);
                publisher.publish::<Candlestick>(REDIS_TOPIC_CANDLESTICK_EXT, &bar);
            }

            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
            prev_bar_time_end = now / INTERVAL * INTERVAL;
            prev_bar_time_begin = prev_bar_time_end - INTERVAL;
            cur_bar_time_end = prev_bar_time_end + INTERVAL;

            cache_prev = cache;
            cache = HashMap::new();
            cache.insert(key, vec![trade_msg]);
        }
    }
}
