use crypto_crawler::*;
use crypto_crawlers::constants::REDIS_TOPIC_TRADE;
use crypto_msg_parser::{parse_trade, TradeMsg};
use log::*;
use std::{
    env,
    str::FromStr,
    sync::{Arc, Mutex},
};
use utils::pubsub::Publisher;

pub fn crawl(exchange: &'static str, market_type: MarketType, redis_url: &'static str) {
    let publisher = Arc::new(Mutex::new(Publisher::new(redis_url)));

    let on_msg_ext = Arc::new(Mutex::new(move |msg: Message| {
        let trades = parse_trade(&msg.exchange, msg.market_type, &msg.json).unwrap();
        for trade in trades.iter() {
            publisher
                .lock()
                .unwrap()
                .publish::<TradeMsg>(REDIS_TOPIC_TRADE, trade);
        }
    }));

    crawl_trade(exchange, market_type, None, on_msg_ext, None);
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: trade_crawler <exchange> <market_type>");
        return;
    }

    let exchange: &'static str = Box::leak(args[1].clone().into_boxed_str());

    let market_type = MarketType::from_str(&args[2]);
    if market_type.is_err() {
        println!("Unknown market type: {}", &args[2]);
        return;
    }
    let market_type = market_type.unwrap();

    let redis_url: &'static str = if std::env::var("REDIS_URL").is_err() {
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

    crawl(exchange, market_type, redis_url);
}
