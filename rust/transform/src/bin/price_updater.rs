use crypto_msg_parser::{MarketType, TradeMsg};
use log::*;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};
use transform::constants::{REDIS_TOPIC_CURRENCY_PRICE, REDIS_TOPIC_TRADE_PARSED};
use utils::wait_redis;

const BETA: f64 = 0.9; // Vt=βVt-1 + (1-β)
const REDIS_TOPIC_CURRENCY_PRICE_CHANNEL: &str = "carbonbot:misc:currency_price_channel";

pub struct PriceUpdater {
    redis_url: String,
    prices: Arc<Mutex<HashMap<String, f64>>>,
    conn: Arc<Mutex<redis::Connection>>,
}

#[derive(Serialize, Deserialize)]
struct CurrencyPrice {
    currency: String,
    price: f64,
}

impl PriceUpdater {
    pub fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).unwrap();
        let conn = client.get_connection().unwrap();

        PriceUpdater {
            redis_url: redis_url.to_string(),
            prices: Arc::new(Mutex::new(HashMap::new())),
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn run(&self) {
        let handle1 = self.subscribe_mark_price();
        let handle2 = self.subscribe_trade();
        let _ = handle1.join();
        let _ = handle2.join();
    }

    // pub fn get_price(&self, currency: &str) -> Option<f64> {
    //     if let Some(price) = self.prices.lock().unwrap().get(currency) {
    //         Some(*price)
    //     } else {
    //         None
    //     }
    // }

    fn subscribe_trade(&self) -> JoinHandle<()> {
        let prices_clone = self.prices.clone();
        let conn_clone = self.conn.clone();
        let redis_url = self.redis_url.to_string();
        thread::spawn(move || {
            let client = redis::Client::open(redis_url).unwrap();
            let mut connection = client.get_connection().unwrap();
            let mut pubsub = connection.as_pubsub();
            pubsub.subscribe(REDIS_TOPIC_TRADE_PARSED).unwrap();

            loop {
                let msg = pubsub.get_message().unwrap();
                let payload: String = msg.get_payload().unwrap();
                let trade_msg = serde_json::from_str::<TradeMsg>(&payload).unwrap();
                match trade_msg.market_type {
                    MarketType::Spot | MarketType::InverseSwap | MarketType::LinearSwap => {
                        let v: Vec<&str> = trade_msg.pair.split('/').collect();
                        let base = v[0];
                        let quote = v[1];
                        if quote == "USD" || quote == "USDT" {
                            Self::update_price(
                                base,
                                trade_msg.price,
                                prices_clone.clone(),
                                conn_clone.clone(),
                            )
                        }
                    }
                    _ => (),
                }
            }
        })
    }

    fn subscribe_mark_price(&self) -> JoinHandle<()> {
        let prices_clone = self.prices.clone();
        let conn_clone = self.conn.clone();
        let redis_url = self.redis_url.to_string();
        thread::spawn(move || {
            let client = redis::Client::open(redis_url).unwrap();
            let mut connection = client.get_connection().unwrap();
            let mut pubsub = connection.as_pubsub();
            pubsub
                .subscribe(REDIS_TOPIC_CURRENCY_PRICE_CHANNEL)
                .unwrap();

            loop {
                let msg = pubsub.get_message().unwrap();
                let payload: String = msg.get_payload().unwrap();
                if let Ok(mark_price) = serde_json::from_str::<CurrencyPrice>(&payload) {
                    Self::update_price(
                        &mark_price.currency,
                        mark_price.price,
                        prices_clone.clone(),
                        conn_clone.clone(),
                    )
                }
            }
        })
    }

    fn update_price(
        currency: &str,
        new_price: f64,
        prices: Arc<Mutex<HashMap<String, f64>>>,
        conn: Arc<Mutex<redis::Connection>>,
    ) {
        let mut guard = prices.lock().unwrap();

        let price_ema = if guard.contains_key(currency) {
            let prev_price = guard.get(currency).unwrap();
            *prev_price * BETA + (1.0 - BETA) * new_price
        } else {
            new_price
        };
        guard.insert(currency.to_string(), price_ema);
        let _ = conn.lock().unwrap().hset::<&str, &str, f64, i64>(
            REDIS_TOPIC_CURRENCY_PRICE,
            currency,
            price_ema,
        );
    }
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

    let updater = PriceUpdater::new(redis_url);
    updater.run();
}
