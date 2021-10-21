use std::{sync::mpsc::Receiver, thread::JoinHandle};

use crypto_msg_parser::{FundingRateMsg, MarketType, MessageType, TradeMsg};
use log::*;
use serde::{Deserialize, Serialize};
use transform::constants::{REDIS_TOPIC_FUNDING_RATE_PARSED, REDIS_TOPIC_TRADE_PARSED};
use utils::{pubsub::Publisher, wait_redis};

const REDIS_TOPIC_TRADE: &str = "carbonbot:trade";
const REDIS_TOPIC_FUNDING_RATE: &str = "carbonbot:funding_rate";

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

fn create_parser_thread(
    thread_name: String,
    rx: Receiver<Message>,
    redis_url: String,
) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let mut publisher = Publisher::new(&redis_url);
            for raw_msg in rx {
                match raw_msg.msg_type {
                    MessageType::Trade => {
                        let trade_msgs = if let Ok(tmp) = crypto_msg_parser::parse_trade(
                            &raw_msg.exchange,
                            raw_msg.market_type,
                            &raw_msg.json,
                        ) {
                            tmp
                        } else {
                            if raw_msg.exchange != "bitmex" {
                                // bitmex has index such as .XTZBON, .XBT, etc.
                                warn!("{}", serde_json::to_string(&raw_msg).unwrap());
                            }
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
                            warn!("{}", serde_json::to_string(&raw_msg).unwrap());
                            vec![]
                        };
                        for rate in rates {
                            publisher
                                .publish::<FundingRateMsg>(REDIS_TOPIC_FUNDING_RATE_PARSED, &rate);
                        }
                    }
                    _ => panic!("unexpected message type {}", raw_msg.msg_type),
                };
            }
        })
        .unwrap()
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

    // subscriber
    let mut connection = {
        let client = redis::Client::open(redis_url).unwrap();
        client.get_connection().unwrap()
    };
    let mut pubsub = connection.as_pubsub();
    pubsub.subscribe(REDIS_TOPIC_TRADE).unwrap();
    pubsub.subscribe(REDIS_TOPIC_FUNDING_RATE).unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<Message>();
    let _ = create_parser_thread("parser".to_string(), rx, redis_url.to_string());
    loop {
        match pubsub.get_message() {
            Ok(msg) => {
                let payload: String = msg.get_payload().unwrap();
                let raw_msg = serde_json::from_str::<Message>(&payload).unwrap();
                tx.send(raw_msg).unwrap();
            }
            Err(err) => error!("{}", err),
        }
    }
}
