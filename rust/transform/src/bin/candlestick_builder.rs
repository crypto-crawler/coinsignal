use crypto_market_type::MarketType;
use crypto_msg_parser::{TradeMsg, TradeSide};
use lazy_static::lazy_static;
use log::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};
use transform::constants::*;
use utils::{pubsub::Publisher, PriceCache};

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

fn calc_volume_usd(trade: &TradeMsg) -> f64 {
    let quote = extract_quote(&trade.pair);
    if STABLE_COINS.contains(quote) {
        trade.price * trade.quantity
    } else if let Some(quote_price) = PRICE_CACHE.get_price(quote) {
        trade.price * trade.quantity * quote_price
    } else {
        panic!("Unknown quote currency {}", quote);
    }
}

fn build_candlestick(
    bar_time: i64,
    bar_size: i64,
    trades: &mut Vec<TradeMsg>,
) -> Option<Candlestick> {
    assert!(!trades.is_empty());
    if trades.is_empty() || !is_good(extract_quote(&trades[0].pair)) {
        return None;
    }

    trades.dedup_by(|a, b| {
        a.timestamp == b.timestamp
            && a.trade_id == b.trade_id
            && a.price == b.price
            && a.quantity == b.quantity
            && a.volume == b.volume
            && a.side == b.side
    });
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
    let volume_quote = trades.iter().map(|x| x.volume * x.price).sum();
    let volume_quote_sell = trades
        .iter()
        .filter(|x| x.side == TradeSide::Sell)
        .map(|x| x.volume * x.price)
        .sum();
    let volume_quote_buy = trades
        .iter()
        .filter(|x| x.side == TradeSide::Buy)
        .map(|x| x.volume * x.price)
        .sum();

    let volume_usd = trades.iter().map(|x| calc_volume_usd(x)).sum();
    let volume_usd_sell = trades
        .iter()
        .filter(|x| x.side == TradeSide::Sell)
        .map(|x| calc_volume_usd(x))
        .sum();
    let volume_usd_buy = trades
        .iter()
        .filter(|x| x.side == TradeSide::Buy)
        .map(|x| calc_volume_usd(x))
        .sum();

    let btc_price = PRICE_CACHE.get_price("BTC").unwrap();

    let candlestick = Candlestick {
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
        volume_usd,
        volume_usd_sell,
        volume_usd_buy,
        volume_btc: volume_usd / btc_price,
        volume_btc_sell: volume_usd_sell / btc_price,
        volume_btc_buy: volume_usd_buy / btc_price,

        vwap: volume_quote / volume,

        count: trades.len() as i64,
        count_sell: trades.iter().filter(|x| x.side == TradeSide::Sell).count() as i64,
        count_buy: trades.iter().filter(|x| x.side == TradeSide::Buy).count() as i64,
    };

    Some(candlestick)
}

fn extract_quote(pair: &str) -> &str {
    let slash_pos = pair.find("/").unwrap();
    &pair[slash_pos + 1..]
}

fn is_good(quote: &str) -> bool {
    STABLE_COINS.contains(quote) || PRICE_CACHE.get_price(quote).is_some()
}

// Merge trades into 1-minute klines
fn main() {
    env_logger::init();
    PRICE_CACHE.wait_until_ready();

    let mut publisher = Publisher::new(*REDIS_URL);

    let mut prev_bar_time_end = -1i64;
    let mut prev_bar_time_begin = -1i64;
    let mut cur_bar_time_end = -1i64;

    let mut cache_prev = HashMap::<String, Vec<TradeMsg>>::new();
    let mut cache = HashMap::<String, Vec<TradeMsg>>::new();

    // subscriber
    let client = redis::Client::open(*REDIS_URL).unwrap();
    let mut connection = client.get_connection().unwrap();
    let mut pubsub = connection.as_pubsub();
    pubsub.subscribe(REDIS_TOPIC_TRADE).unwrap();

    let start_time = Instant::now();
    loop {
        let msg = pubsub.get_message().unwrap();
        let payload: String = msg.get_payload().unwrap();
        let trade_msg = serde_json::from_str::<TradeMsg>(&payload).unwrap();

        if !is_good(extract_quote(&trade_msg.pair)) {
            // warn!(
            //     "{}, {}, {}",
            //     trade_msg.exchange, trade_msg.market_type, trade_msg.pair
            // );
            continue;
        }

        let key = format!(
            "{}-{}-{}-{}",
            trade_msg.exchange, trade_msg.market_type, trade_msg.pair, trade_msg.symbol
        );

        if prev_bar_time_end == -1 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            prev_bar_time_end = now / INTERVAL * INTERVAL;
            prev_bar_time_begin = prev_bar_time_end - INTERVAL;
            cur_bar_time_end = prev_bar_time_end + INTERVAL;
        }

        if trade_msg.timestamp < prev_bar_time_begin {
            // Don't log old messages at the beginning 5 minutes
            if start_time.elapsed().as_secs() > 300 {
                warn!(
                    "Expired msg, prev_bar_time_begin: {}, trade_msg: {}",
                    prev_bar_time_begin,
                    serde_json::to_string(&trade_msg).unwrap()
                );
            }
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
                if let Some(bar) = build_candlestick(prev_bar_time_end, INTERVAL, trades) {
                    publisher.publish::<Candlestick>(REDIS_TOPIC_CANDLESTICK_EXT, &bar);
                }
            }

            prev_bar_time_begin = prev_bar_time_end;
            prev_bar_time_end = cur_bar_time_end;
            cur_bar_time_end = cur_bar_time_end + INTERVAL;

            cache_prev = cache;
            cache = HashMap::new();
            cache.insert(key, vec![trade_msg]);
        }
    }
}
