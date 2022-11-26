use log::*;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct PriceCache {
    redis_url: String,
    prices: Arc<Mutex<HashMap<String, f64>>>,
}

#[derive(Serialize, Deserialize)]
struct MarkPrice {
    currency: String,
    price: f64,
}

impl PriceCache {
    pub fn new(redis_url: &str) -> Self {
        let cache = PriceCache {
            redis_url: redis_url.to_string(),
            prices: Arc::new(Mutex::new(HashMap::new())),
        };

        cache.run(); // retreive prices per 3 seconds

        cache
    }

    pub fn wait_until_ready(&self) {
        loop {
            let ready = self.is_ready();
            if ready {
                break;
            } else {
                info!("price cache is not ready yet");
                std::thread::sleep(Duration::from_secs(3));
            }
        }
    }

    pub fn get_price(&self, currency: &str) -> Option<f64> {
        self.prices.lock().unwrap().get(currency).copied()
    }

    fn run(&self) {
        let prices_clone = self.prices.clone();
        let redis_url = self.redis_url.to_string();
        thread::spawn(move || {
            let client = redis::Client::open(redis_url).unwrap();
            let mut conn = client.get_connection().unwrap();

            loop {
                if let Ok(map) =
                    conn.hgetall::<&str, HashMap<String, f64>>("coinsignal:currency_price")
                {
                    let mut guard = prices_clone.lock().unwrap();
                    for (k, v) in map.iter() {
                        guard.insert(k.clone(), *v);
                    }
                }
                std::thread::sleep(Duration::from_secs(3));
            }
        });
    }

    fn is_ready(&self) -> bool {
        let hot_coins: [&str; 15] = [
            "BTC", "ETH", "BNB", "XRP", "DOGE", "ADA", "MATIC", "LTC", "DOT", "SOL", "ATOM", "UNI",
            "AVAX", "LINK", "BCH",
        ];
        for &coin in hot_coins.iter() {
            if !self.prices.lock().unwrap().contains_key(coin) {
                return false;
            }
        }
        true
    }
}
