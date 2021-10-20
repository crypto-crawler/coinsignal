use crypto_msg_parser::{FundingRateMsg, MarketType, MessageType, TradeMsg};
use log::*;
use serde::{Deserialize, Serialize};
use transform::constants::{
    REDIS_TOPIC_FUNDING_RATE, REDIS_TOPIC_FUNDING_RATE_PARSED, REDIS_TOPIC_TRADE,
    REDIS_TOPIC_TRADE_PARSED,
};
use utils::{pubsub::Publisher, wait_redis};

/// Message represents messages received by crawlers.
#[derive(Serialize, Deserialize)]
pub struct Message {
    /// The exchange name, unique for each exchage
    pub exchange: String,
    /// Market type
    pub market_type: MarketType,
    /// Message type
    pub msg_type: MessageType,
    /// Unix timestamp in milliseconds
    pub received_at: u64,
    /// the original message
    pub json: String,
}

fn main() {
    env_logger::init();
    let redis_url = if std::env::var("REDIS_URL").is_err() {
        info!(
            "The REDIS_URL environment variable is empty, using redis://localhost:6379 by default"
        );
        "redis://localhost:6379"
    } else {
        let url = std::env::var("REDIS_URL").unwrap();
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
    pubsub.subscribe(REDIS_TOPIC_FUNDING_RATE).unwrap();

    loop {
        let ret = pubsub.get_message();
        if ret.is_err() {
            warn!("{}", ret.err().unwrap());
            continue;
        }
        let msg = ret.unwrap();
        let payload: String = msg.get_payload().unwrap();
        let raw_msg = serde_json::from_str::<Message>(&payload).unwrap();
        match raw_msg.msg_type {
            MessageType::Trade => {
                let trade_msgs = if let Ok(tmp) = crypto_msg_parser::parse_trade(
                    &raw_msg.exchange,
                    raw_msg.market_type,
                    &raw_msg.json,
                ) {
                    tmp
                } else {
                    vec![]
                };
                for trade_msg in trade_msgs {
                    publisher.publish::<TradeMsg>(REDIS_TOPIC_TRADE_PARSED, &trade_msg);
                }
            }
            MessageType::FundingRate => {
                let rates = if let Ok(tmp) = crypto_msg_parser::parse_funding_rate(
                    &raw_msg.exchange,
                    raw_msg.market_type,
                    &raw_msg.json,
                ) {
                    tmp
                } else {
                    vec![]
                };
                for rate in rates {
                    publisher.publish::<FundingRateMsg>(REDIS_TOPIC_FUNDING_RATE_PARSED, &rate);
                }
            }
            _ => panic!("unexpected message type"),
        };
    }
}
